use std::{collections::BTreeSet, path::PathBuf, thread, time::Duration};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use super::{
    clock::{monotonic_now_ms, unix_now_ms},
    manager::{write_pending_kbdrec, RecordingManager},
};

pub const RECORDING_UI_EVENT: &str = "recording-ui";
const OVERLAY_SYNC_FEEDBACK_EVENT: &str = "overlay-sync-feedback";
const COUNTDOWN_SECONDS: u32 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordingHotkeyMode {
    Disabled,
    Toggle,
    Separate,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingHotkeyConfig {
    pub mode: RecordingHotkeyMode,
    pub start: Vec<String>,
    pub stop: Vec<String>,
    pub sync: Vec<String>,
}

impl Default for RecordingHotkeyConfig {
    fn default() -> Self {
        Self {
            mode: RecordingHotkeyMode::Toggle,
            start: vec![
                "ctrl-left".to_string(),
                "shift-left".to_string(),
                "r".to_string(),
            ],
            stop: vec![
                "ctrl-left".to_string(),
                "shift-left".to_string(),
                "r".to_string(),
            ],
            sync: vec!["f8".to_string()],
        }
    }
}

impl RecordingHotkeyConfig {
    pub fn normalized(self) -> Self {
        Self {
            mode: self.mode,
            start: normalize_hotkey(self.start),
            stop: normalize_hotkey(self.stop),
            sync: normalize_hotkey(self.sync),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingRuntimeSettings {
    pub hotkeys: RecordingHotkeyConfig,
    pub output_directory: String,
    pub filename_template: String,
    pub profile_name: String,
    pub fps: u16,
    pub silent: bool,
    pub sync_feedback_enabled: bool,
    pub sync_feedback_duration_ms: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HotkeyAction {
    Start,
    Stop { suppress: Vec<String> },
    Sync { suppress: Vec<String> },
    CancelCountdown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HotkeyMatch {
    pub action: Option<HotkeyAction>,
    pub signature: String,
}

#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum RecordingUiEvent {
    Countdown { remaining: u32, fps: u16 },
    Started { fps: u16, silent: bool },
    Stopped { path: String, silent: bool },
    Sync,
    CountdownCancelled,
    Error { message: String },
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OverlaySyncFeedbackPayload {
    duration_ms: u32,
}

pub fn evaluate_hotkeys(app_handle: &AppHandle, active_keys: &[String]) {
    let Some(manager) = app_handle.try_state::<RecordingManager>() else {
        return;
    };
    let Some(action) = manager.take_hotkey_action(active_keys) else {
        return;
    };

    match action {
        HotkeyAction::Start => start_countdown(app_handle.clone()),
        HotkeyAction::Stop { suppress } => stop_from_hotkey(app_handle, suppress),
        HotkeyAction::Sync { suppress } => sync_from_hotkey(app_handle, suppress),
        HotkeyAction::CancelCountdown => {
            manager.cancel_countdown();
            emit_ui(app_handle, RecordingUiEvent::CountdownCancelled);
        }
    }
}

pub fn match_recording_hotkey(
    config: &RecordingHotkeyConfig,
    active_keys: &BTreeSet<String>,
    last_signature: &str,
    is_recording: bool,
    is_counting_down: bool,
) -> HotkeyMatch {
    let signature = hotkey_signature(active_keys);
    if signature == last_signature {
        return HotkeyMatch {
            action: None,
            signature,
        };
    }

    let matches_start = is_hotkey_match(active_keys, &config.start);
    let matches_stop = is_hotkey_match(active_keys, &config.stop);
    let matches_sync = is_hotkey_match(active_keys, &config.sync);

    if !matches_start && !matches_stop && !matches_sync {
        return HotkeyMatch {
            action: None,
            signature: if signature.is_empty() {
                String::new()
            } else {
                last_signature.to_string()
            },
        };
    }

    if matches_sync && is_recording {
        return HotkeyMatch {
            action: Some(HotkeyAction::Sync {
                suppress: config.sync.clone(),
            }),
            signature,
        };
    }

    if config.mode == RecordingHotkeyMode::Disabled {
        return HotkeyMatch {
            action: None,
            signature: last_signature.to_string(),
        };
    }

    if config.mode == RecordingHotkeyMode::Toggle {
        if is_counting_down {
            return HotkeyMatch {
                action: Some(HotkeyAction::CancelCountdown),
                signature,
            };
        }
        if is_recording {
            return HotkeyMatch {
                action: Some(HotkeyAction::Stop {
                    suppress: config.stop.clone(),
                }),
                signature,
            };
        }
        return HotkeyMatch {
            action: Some(HotkeyAction::Start),
            signature,
        };
    }

    if !is_recording && matches_start {
        return HotkeyMatch {
            action: Some(HotkeyAction::Start),
            signature,
        };
    }
    if is_recording && matches_stop {
        return HotkeyMatch {
            action: Some(HotkeyAction::Stop {
                suppress: config.stop.clone(),
            }),
            signature,
        };
    }

    HotkeyMatch {
        action: None,
        signature: last_signature.to_string(),
    }
}

pub fn normalize_hotkey(keys: impl IntoIterator<Item = impl AsRef<str>>) -> Vec<String> {
    let mut normalized = keys
        .into_iter()
        .map(|key| key.as_ref().to_string())
        .collect::<Vec<_>>();
    normalized.sort_by(|left, right| {
        hotkey_priority(left)
            .cmp(&hotkey_priority(right))
            .then_with(|| left.cmp(right))
    });
    normalized.dedup();
    normalized
}

pub fn is_hotkey_match(active_keys: &BTreeSet<String>, hotkey: &[String]) -> bool {
    let normalized = normalize_hotkey(hotkey.iter());
    !normalized.is_empty() && normalized.iter().all(|key| active_keys.contains(key))
}

pub fn hotkey_signature(active_keys: &BTreeSet<String>) -> String {
    normalize_hotkey(active_keys.iter()).join("+")
}

fn hotkey_priority(key: &str) -> u8 {
    if key.starts_with("ctrl-") {
        0
    } else if key.starts_with("shift-") {
        1
    } else if key.starts_with("alt-") {
        2
    } else if key.starts_with("meta-") {
        3
    } else {
        4
    }
}

fn start_countdown(app_handle: AppHandle) {
    let Some((generation, fps, silent)) = app_handle
        .try_state::<RecordingManager>()
        .and_then(|manager| manager.begin_countdown())
    else {
        return;
    };

    emit_ui(
        &app_handle,
        RecordingUiEvent::Countdown {
            remaining: COUNTDOWN_SECONDS,
            fps,
        },
    );

    thread::spawn(move || {
        let Some(manager) = app_handle.try_state::<RecordingManager>() else {
            return;
        };
        for remaining in (0..COUNTDOWN_SECONDS).rev() {
            thread::sleep(Duration::from_secs(1));
            if !manager.countdown_is(generation) {
                return;
            }
            if remaining == 0 {
                break;
            }
            emit_ui(&app_handle, RecordingUiEvent::Countdown { remaining, fps });
        }

        if !manager.finish_countdown(generation) {
            return;
        }

        let start_unix_ms = match unix_now_ms() {
            Ok(value) => value,
            Err(message) => {
                emit_ui(&app_handle, RecordingUiEvent::Error { message });
                return;
            }
        };

        if let Err(message) = manager.start(fps, start_unix_ms, monotonic_now_ms()) {
            emit_ui(&app_handle, RecordingUiEvent::Error { message });
            return;
        }

        emit_ui(&app_handle, RecordingUiEvent::Started { fps, silent });
    });
}

fn stop_from_hotkey(app_handle: &AppHandle, suppress: Vec<String>) {
    let Some(manager) = app_handle.try_state::<RecordingManager>() else {
        return;
    };
    let _ = manager.suppress_recent_keys(suppress);

    let settings = match manager.snapshot_stop_settings() {
        Ok(settings) => settings,
        Err(message) => {
            emit_ui(app_handle, RecordingUiEvent::Error { message });
            return;
        }
    };

    let output_dir = if settings.output_directory.trim().is_empty() {
        match fallback_output_dir(app_handle) {
            Ok(path) => path,
            Err(message) => {
                emit_ui(app_handle, RecordingUiEvent::Error { message });
                return;
            }
        }
    } else {
        PathBuf::from(settings.output_directory)
    };

    let now_ms = match unix_now_ms() {
        Ok(value) => value,
        Err(message) => {
            emit_ui(app_handle, RecordingUiEvent::Error { message });
            return;
        }
    };

    let pending = match manager.take_pending_kbdrec(
        output_dir,
        now_ms,
        &settings.filename_template,
        &settings.profile_name,
        settings.fps,
    ) {
        Ok(pending) => pending,
        Err(message) if message.contains("has not started") => return,
        Err(message) => {
            emit_ui(app_handle, RecordingUiEvent::Error { message });
            return;
        }
    };

    // Encode + disk I/O on a writer thread so ingest can keep applying edges.
    let silent = settings.silent;
    let writer_handle = app_handle.clone();
    if let Err(error) = thread::Builder::new()
        .name("kbdrec-write".into())
        .spawn(move || match write_pending_kbdrec(pending) {
            Ok(result) => emit_ui(
                &writer_handle,
                RecordingUiEvent::Stopped {
                    path: result.path,
                    silent,
                },
            ),
            Err(message) => emit_ui(&writer_handle, RecordingUiEvent::Error { message }),
        })
    {
        emit_ui(
            app_handle,
            RecordingUiEvent::Error {
                message: format!("failed to start kbdrec writer: {error}"),
            },
        );
    }
}

fn sync_from_hotkey(app_handle: &AppHandle, suppress: Vec<String>) {
    let Some(manager) = app_handle.try_state::<RecordingManager>() else {
        return;
    };
    let _ = manager.suppress_recent_keys(suppress);
    if let Err(message) = manager.add_marker(monotonic_now_ms(), "sync") {
        emit_ui(app_handle, RecordingUiEvent::Error { message });
        return;
    }

    if let Some((enabled, duration_ms)) = manager.sync_feedback() {
        if enabled {
            let _ = app_handle.emit_to(
                "pov",
                OVERLAY_SYNC_FEEDBACK_EVENT,
                OverlaySyncFeedbackPayload { duration_ms },
            );
        }
    }

    emit_ui(app_handle, RecordingUiEvent::Sync);
}

fn emit_ui(app_handle: &AppHandle, event: RecordingUiEvent) {
    let _ = app_handle.emit(RECORDING_UI_EVENT, event);
}

fn fallback_output_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
    #[cfg(debug_assertions)]
    {
        return app_handle
            .path()
            .app_config_dir()
            .map(|path| path.join("recording-files"))
            .map_err(|error| error.to_string());
    }

    #[cfg(not(debug_assertions))]
    {
        let _ = app_handle;
        let executable = std::env::current_exe().map_err(|error| error.to_string())?;
        executable
            .parent()
            .map(|path| path.join("recording-files"))
            .ok_or_else(|| "failed to resolve executable directory".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        is_hotkey_match, match_recording_hotkey, normalize_hotkey, HotkeyAction,
        RecordingHotkeyConfig, RecordingHotkeyMode,
    };
    use std::collections::BTreeSet;

    fn keys(ids: &[&str]) -> BTreeSet<String> {
        ids.iter().map(|id| (*id).to_string()).collect()
    }

    fn toggle_config() -> RecordingHotkeyConfig {
        RecordingHotkeyConfig::default()
    }

    #[test]
    fn normalizes_ctrl_shift_r_like_frontend() {
        assert_eq!(
            normalize_hotkey(["r", "ctrl-left", "r", "shift-left"]),
            vec!["ctrl-left", "shift-left", "r"]
        );
    }

    #[test]
    fn matches_subset_chords() {
        assert!(is_hotkey_match(
            &keys(&["ctrl-left", "shift-left", "r", "w"]),
            &[
                "shift-left".to_string(),
                "ctrl-left".to_string(),
                "r".to_string()
            ],
        ));
        assert!(!is_hotkey_match(
            &keys(&["ctrl-left"]),
            &["ctrl-left".to_string(), "r".to_string()],
        ));
    }

    #[test]
    fn f8_adds_sync_marker_while_recording() {
        let matched = match_recording_hotkey(&toggle_config(), &keys(&["f8"]), "", true, false);
        assert_eq!(
            matched.action,
            Some(HotkeyAction::Sync {
                suppress: vec!["f8".to_string()],
            })
        );
    }

    #[test]
    fn same_signature_does_not_retrigger() {
        let matched = match_recording_hotkey(&toggle_config(), &keys(&["f8"]), "f8", true, false);
        assert_eq!(matched.action, None);
        assert_eq!(matched.signature, "f8");
    }

    #[test]
    fn empty_active_set_clears_signature() {
        let matched = match_recording_hotkey(&toggle_config(), &keys(&[]), "f8", true, false);
        assert_eq!(matched.action, None);
        assert_eq!(matched.signature, "");
    }

    #[test]
    fn toggle_starts_from_ctrl_shift_r() {
        let matched = match_recording_hotkey(
            &toggle_config(),
            &keys(&["ctrl-left", "shift-left", "r"]),
            "",
            false,
            false,
        );
        assert_eq!(matched.action, Some(HotkeyAction::Start));
    }

    #[test]
    fn toggle_cancels_countdown() {
        let matched = match_recording_hotkey(
            &toggle_config(),
            &keys(&["ctrl-left", "shift-left", "r"]),
            "",
            false,
            true,
        );
        assert_eq!(matched.action, Some(HotkeyAction::CancelCountdown));
    }

    #[test]
    fn toggle_stops_while_recording() {
        let matched = match_recording_hotkey(
            &toggle_config(),
            &keys(&["ctrl-left", "shift-left", "r"]),
            "",
            true,
            false,
        );
        assert_eq!(
            matched.action,
            Some(HotkeyAction::Stop {
                suppress: vec![
                    "ctrl-left".to_string(),
                    "shift-left".to_string(),
                    "r".to_string(),
                ],
            })
        );
    }

    #[test]
    fn separate_mode_requires_matching_start_or_stop() {
        let config = RecordingHotkeyConfig {
            mode: RecordingHotkeyMode::Separate,
            start: vec![
                "ctrl-left".to_string(),
                "shift-left".to_string(),
                "r".to_string(),
            ],
            stop: vec![
                "ctrl-left".to_string(),
                "shift-left".to_string(),
                "t".to_string(),
            ],
            sync: vec!["f8".to_string()],
        };

        assert_eq!(
            match_recording_hotkey(
                &config,
                &keys(&["ctrl-left", "shift-left", "r"]),
                "",
                false,
                false
            )
            .action,
            Some(HotkeyAction::Start)
        );
        assert_eq!(
            match_recording_hotkey(
                &config,
                &keys(&["ctrl-left", "shift-left", "t"]),
                "",
                true,
                false
            )
            .action,
            Some(HotkeyAction::Stop {
                suppress: vec![
                    "ctrl-left".to_string(),
                    "shift-left".to_string(),
                    "t".to_string(),
                ],
            })
        );
        assert_eq!(
            match_recording_hotkey(
                &config,
                &keys(&["ctrl-left", "shift-left", "r"]),
                "",
                true,
                false
            )
            .action,
            None
        );
    }

    #[test]
    fn disabled_mode_still_allows_sync() {
        let config = RecordingHotkeyConfig {
            mode: RecordingHotkeyMode::Disabled,
            start: vec!["f9".to_string()],
            stop: vec!["f9".to_string()],
            sync: vec!["f8".to_string()],
        };

        assert_eq!(
            match_recording_hotkey(&config, &keys(&["f8"]), "", true, false).action,
            Some(HotkeyAction::Sync {
                suppress: vec!["f8".to_string()],
            })
        );
        assert_eq!(
            match_recording_hotkey(&config, &keys(&["f8"]), "", false, false).action,
            None
        );
    }

    #[test]
    fn partial_chord_does_not_match() {
        let matched = match_recording_hotkey(
            &toggle_config(),
            &keys(&["ctrl-left", "shift-left"]),
            "",
            false,
            false,
        );
        assert_eq!(matched.action, None);
    }
}
