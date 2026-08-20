use std::{collections::BTreeSet, path::PathBuf, sync::Mutex};

use super::{
    binary,
    filename::format_recording_file_name,
    hotkeys::{match_recording_hotkey, RecordingHotkeyConfig, RecordingRuntimeSettings},
    session::RecordingSession,
    types::StopRecordingResult,
};

pub struct RecordingManager {
    pub(crate) session: Mutex<Option<ActiveRecordingSession>>,
    control: Mutex<RecordingControlState>,
}

struct RecordingControlState {
    hotkeys: RecordingHotkeyConfig,
    active_hotkeys: Option<RecordingHotkeyConfig>,
    output_directory: String,
    filename_template: String,
    profile_name: String,
    fps: u16,
    silent: bool,
    sync_feedback_enabled: bool,
    sync_feedback_duration_ms: u32,
    last_signature: String,
    countdown_generation: u64,
    counting_down: bool,
    is_recording: bool,
}

impl Default for RecordingControlState {
    fn default() -> Self {
        Self {
            hotkeys: RecordingHotkeyConfig::default(),
            active_hotkeys: None,
            output_directory: String::new(),
            filename_template: "${start}-${end}".to_string(),
            profile_name: String::new(),
            fps: 60,
            silent: false,
            sync_feedback_enabled: true,
            sync_feedback_duration_ms: 420,
            last_signature: String::new(),
            countdown_generation: 0,
            counting_down: false,
            is_recording: false,
        }
    }
}

pub(crate) struct RecordingStopSettings {
    pub output_directory: String,
    pub filename_template: String,
    pub profile_name: String,
    pub fps: u16,
    pub silent: bool,
}

pub(crate) struct ActiveRecordingSession {
    pub(crate) start_unix_ms: u64,
    pub(crate) session: RecordingSession,
}

impl RecordingManager {
    pub fn new() -> Self {
        Self {
            session: Mutex::new(None),
            control: Mutex::new(RecordingControlState::default()),
        }
    }

    pub fn apply_runtime_settings(&self, settings: RecordingRuntimeSettings) -> Result<(), String> {
        let mut control = self.control.lock().map_err(|error| error.to_string())?;
        control.hotkeys = settings.hotkeys.normalized();
        control.output_directory = settings.output_directory;
        control.filename_template = if settings.filename_template.trim().is_empty() {
            "${start}-${end}".to_string()
        } else {
            settings.filename_template
        };
        control.profile_name = settings.profile_name;
        control.fps = if settings.fps == 0 { 60 } else { settings.fps };
        control.silent = settings.silent;
        control.sync_feedback_enabled = settings.sync_feedback_enabled;
        control.sync_feedback_duration_ms = settings.sync_feedback_duration_ms.max(100);
        Ok(())
    }

    pub fn take_hotkey_action(
        &self,
        active_keys: &[String],
    ) -> Option<super::hotkeys::HotkeyAction> {
        let mut control = self.control.lock().ok()?;
        let active = active_keys.iter().cloned().collect::<BTreeSet<_>>();
        let matched = match_recording_hotkey(
            control.active_hotkeys.as_ref().unwrap_or(&control.hotkeys),
            &active,
            &control.last_signature,
            control.is_recording,
            control.counting_down,
        );
        control.last_signature = matched.signature;
        matched.action
    }

    pub fn begin_countdown(&self) -> Option<(u64, u16, bool)> {
        let mut control = self.control.lock().ok()?;
        if control.is_recording || control.counting_down {
            return None;
        }
        control.counting_down = true;
        control.countdown_generation = control.countdown_generation.saturating_add(1);
        Some((control.countdown_generation, control.fps, control.silent))
    }

    pub fn countdown_is(&self, generation: u64) -> bool {
        self.control
            .lock()
            .map(|control| control.counting_down && control.countdown_generation == generation)
            .unwrap_or(false)
    }

    pub fn finish_countdown(&self, generation: u64) -> bool {
        let Ok(mut control) = self.control.lock() else {
            return false;
        };
        if !control.counting_down || control.countdown_generation != generation {
            return false;
        }
        control.counting_down = false;
        true
    }

    pub fn cancel_countdown(&self) {
        if let Ok(mut control) = self.control.lock() {
            control.counting_down = false;
            control.countdown_generation = control.countdown_generation.saturating_add(1);
        }
    }

    pub fn snapshot_stop_settings(&self) -> Result<RecordingStopSettings, String> {
        let control = self.control.lock().map_err(|error| error.to_string())?;
        Ok(RecordingStopSettings {
            output_directory: control.output_directory.clone(),
            filename_template: control.filename_template.clone(),
            profile_name: control.profile_name.clone(),
            fps: if control.fps == 0 { 60 } else { control.fps },
            silent: control.silent,
        })
    }

    pub fn sync_feedback(&self) -> Option<(bool, u32)> {
        self.control.lock().ok().map(|control| {
            (
                control.sync_feedback_enabled,
                control.sync_feedback_duration_ms,
            )
        })
    }

    /// Tauri State 持有的线程安全入口；具体按键事件去重由 RecordingSession 负责。
    pub fn start(
        &self,
        fps: u16,
        start_unix_ms: u64,
        start_monotonic_ms: u64,
    ) -> Result<(), String> {
        {
            let mut control = self.control.lock().map_err(|error| error.to_string())?;
            control.counting_down = false;
            control.countdown_generation = control.countdown_generation.saturating_add(1);
        }

        let mut session = self.session.lock().map_err(|error| error.to_string())?;
        if session.is_some() {
            drop(session);
            let mut control = self.control.lock().map_err(|error| error.to_string())?;
            control.is_recording = true;
            return Ok(());
        }

        *session = Some(ActiveRecordingSession {
            start_unix_ms,
            session: RecordingSession::new(fps, start_monotonic_ms),
        });
        drop(session);

        let mut control = self.control.lock().map_err(|error| error.to_string())?;
        control.is_recording = true;
        control.active_hotkeys = Some(control.hotkeys.clone());
        Ok(())
    }

    pub fn record_input(
        &self,
        now_ms: u64,
        key_id: impl Into<String>,
        pressed: bool,
    ) -> Result<(), String> {
        let mut session = self.session.lock().map_err(|error| error.to_string())?;

        if let Some(active_session) = session.as_mut() {
            active_session.session.record_input(now_ms, key_id, pressed);
        }

        Ok(())
    }

    pub fn add_marker(&self, now_ms: u64, name: impl Into<String>) -> Result<(), String> {
        let mut session = self.session.lock().map_err(|error| error.to_string())?;

        if let Some(active_session) = session.as_mut() {
            active_session.session.add_marker(now_ms, name);
        }

        Ok(())
    }

    pub fn suppress_recent_keys(&self, key_ids: Vec<String>) -> Result<(), String> {
        let mut session = self.session.lock().map_err(|error| error.to_string())?;

        if let Some(active_session) = session.as_mut() {
            active_session.session.suppress_recent_keys(&key_ids);
        }

        Ok(())
    }

    #[cfg(test)]
    pub fn stop(&self, output_dir: PathBuf, now_ms: u64) -> Result<StopRecordingResult, String> {
        self.stop_with_filename_template(output_dir, now_ms, "${start}-${end}", "", 0)
    }

    /// Take the in-memory session and mark recording stopped. Does not encode
    /// or touch the disk — that belongs on the kbdrec writer thread.
    pub(crate) fn take_pending_kbdrec(
        &self,
        output_dir: PathBuf,
        now_ms: u64,
        filename_template: &str,
        profile_name: &str,
        fps: u16,
    ) -> Result<PendingKbdrecWrite, String> {
        let mut session = self.session.lock().map_err(|error| error.to_string())?;
        let Some(active_session) = session.take() else {
            return Err("recording has not started".to_string());
        };
        drop(session);

        if let Ok(mut control) = self.control.lock() {
            control.is_recording = false;
            control.active_hotkeys = None;
        }

        Ok(PendingKbdrecWrite {
            active_session,
            output_dir,
            now_ms,
            filename_template: filename_template.to_string(),
            profile_name: profile_name.to_string(),
            fps,
        })
    }

    pub fn stop_with_filename_template(
        &self,
        output_dir: PathBuf,
        now_ms: u64,
        filename_template: &str,
        profile_name: &str,
        fps: u16,
    ) -> Result<StopRecordingResult, String> {
        let pending =
            self.take_pending_kbdrec(output_dir, now_ms, filename_template, profile_name, fps)?;
        write_pending_kbdrec(pending)
    }
}

/// Session payload for the dedicated file writer. Ingest/capture must not
/// encode or `fs::write` this.
pub(crate) struct PendingKbdrecWrite {
    active_session: ActiveRecordingSession,
    output_dir: PathBuf,
    now_ms: u64,
    filename_template: String,
    profile_name: String,
    fps: u16,
}

pub(crate) fn write_pending_kbdrec(
    pending: PendingKbdrecWrite,
) -> Result<StopRecordingResult, String> {
    std::fs::create_dir_all(&pending.output_dir).map_err(|error| error.to_string())?;

    let file_name = format_recording_file_name(
        &pending.filename_template,
        pending.active_session.start_unix_ms,
        pending.now_ms,
        &pending.profile_name,
        if pending.fps == 0 {
            pending.active_session.session.fps
        } else {
            pending.fps
        },
    );
    let path = pending.output_dir.join(file_name);
    let contents = binary::encode_kbdrec(&pending.active_session.session.snapshot())?;
    std::fs::write(&path, contents).map_err(|error| error.to_string())?;

    Ok(StopRecordingResult {
        path: path.to_string_lossy().to_string(),
    })
}
