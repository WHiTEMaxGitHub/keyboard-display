pub mod mapping;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod unsupported;

use serde::Serialize;
use std::{collections::BTreeSet, sync::Mutex};
use tauri::{AppHandle, Emitter, Manager};

use crate::debug_log;

const INPUT_STATE_EVENT: &str = "input-state";
const OVERLAY_ACTIVE_KEYS_EVENT: &str = "overlay-active-keys";

#[derive(Clone, Serialize)]
pub struct InputStatePayload {
    #[serde(rename = "keyId")]
    pub key_id: String,
    pub pressed: bool,
}

#[derive(Clone, Serialize)]
pub struct OverlayActiveKeysPayload {
    #[serde(rename = "keyIds")]
    pub key_ids: Vec<String>,
}

pub struct InputStateBridge {
    active_keys: Mutex<BTreeSet<String>>,
}

impl InputStateBridge {
    pub fn new() -> Self {
        Self {
            active_keys: Mutex::new(BTreeSet::new()),
        }
    }

    pub fn update(
        &self,
        app_handle: &AppHandle,
        key_id: String,
        pressed: bool,
    ) -> Result<(), String> {
        let key_ids = self.apply_key(&key_id, pressed)?;
        log_input_debug(&key_id, pressed, &key_ids);

        let payload = OverlayActiveKeysPayload { key_ids };
        app_handle
            .emit_to("pov", OVERLAY_ACTIVE_KEYS_EVENT, payload)
            .map_err(|error| error.to_string())
    }

    /// Overlay-set mutation used by `emit_input_state` via `update`.
    fn apply_key(&self, key_id: &str, pressed: bool) -> Result<Vec<String>, String> {
        let mut active_keys = self.active_keys.lock().map_err(|error| error.to_string())?;
        if pressed {
            active_keys.insert(key_id.to_string());
        } else {
            active_keys.remove(key_id);
        }
        Ok(active_keys.iter().cloned().collect())
    }

    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    pub(super) fn is_active(&self, key_id: &str) -> bool {
        self.active_keys
            .lock()
            .map(|keys| keys.contains(key_id))
            .unwrap_or(false)
    }

    #[cfg(test)]
    fn snapshot(&self) -> Vec<String> {
        self.active_keys
            .lock()
            .map(|keys| keys.iter().cloned().collect())
            .unwrap_or_default()
    }
}

pub fn start_native_input_backend(app_handle: AppHandle) {
    #[cfg(target_os = "macos")]
    macos::start(app_handle);

    #[cfg(target_os = "windows")]
    windows::start(app_handle);

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    unsupported::start(app_handle);
}

fn emit_input_state(app_handle: &AppHandle, key_id: impl Into<String>, pressed: bool) {
    let key_id = key_id.into();
    let payload = InputStatePayload {
        key_id: key_id.clone(),
        pressed,
    };

    if let Some(state) = app_handle.try_state::<InputStateBridge>() {
        if let Err(error) = state.update(app_handle, key_id, pressed) {
            debug_log::error(
                "input",
                &format!("failed to update overlay input state: {error}"),
            );
        }
    }

    if let Err(error) = app_handle.emit(INPUT_STATE_EVENT, payload) {
        debug_log::error(
            "input",
            &format!("failed to emit input state to config: {error}"),
        );
    }
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn emit_backend_log(
    _app_handle: &AppHandle,
    message: impl Into<String>,
    details: impl IntoIterator<Item = (impl Into<String>, String)>,
) {
    let message = message.into();
    let details = details
        .into_iter()
        .map(|(key, value)| (key.into(), value))
        .collect::<std::collections::BTreeMap<String, String>>();

    debug_log::warn("input-backend", &format!("{message} {details:?}"));
}

fn log_input_debug(key_id: &str, pressed: bool, active_keys: &[String]) {
    if !debug_log::input_debug_enabled() {
        return;
    }
    debug_log::debug(
        "input",
        &format!("key_id={key_id} pressed={pressed} active_keys={active_keys:?}"),
    );
}

#[cfg(test)]
mod tests {
    use super::InputStateBridge;
    use std::{thread, time::Duration};

    fn keys(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|id| (*id).to_string()).collect()
    }

    #[test]
    fn jump_throw_sequence_clears_mouse_left_and_space() {
        let bridge = InputStateBridge::new();

        assert_eq!(
            bridge.apply_key("mouse-left", true).unwrap(),
            keys(&["mouse-left"])
        );
        assert_eq!(
            bridge.apply_key("space", true).unwrap(),
            keys(&["mouse-left", "space"])
        );
        assert_eq!(
            bridge.apply_key("space", false).unwrap(),
            keys(&["mouse-left"])
        );

        thread::sleep(Duration::from_millis(80));

        assert_eq!(bridge.apply_key("mouse-left", false).unwrap(), keys(&[]));
        assert!(bridge.snapshot().is_empty());
        assert!(!bridge.is_active("mouse-left"));
        assert!(!bridge.is_active("space"));
    }
}
