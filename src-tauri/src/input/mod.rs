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

const INPUT_STATE_EVENT: &str = "input-state";
const OVERLAY_ACTIVE_KEYS_EVENT: &str = "overlay-active-keys";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const INPUT_BACKEND_LOG_EVENT: &str = "input-backend-log";

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

#[derive(Clone, Serialize)]
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub struct InputBackendLogPayload {
    pub message: String,
    pub details: std::collections::BTreeMap<String, String>,
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
        let mut active_keys = self.active_keys.lock().map_err(|error| error.to_string())?;
        if pressed {
            active_keys.insert(key_id);
        } else {
            active_keys.remove(&key_id);
        }

        let payload = OverlayActiveKeysPayload {
            key_ids: active_keys.iter().cloned().collect(),
        };
        app_handle
            .emit_to("pov", OVERLAY_ACTIVE_KEYS_EVENT, payload)
            .map_err(|error| error.to_string())
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
            eprintln!("failed to update overlay input state: {error}");
        }
    }

    if let Err(error) = app_handle.emit_to("config", INPUT_STATE_EVENT, payload) {
        eprintln!("failed to emit input state to config: {error}");
    }
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn emit_backend_log(
    app_handle: &AppHandle,
    message: impl Into<String>,
    details: impl IntoIterator<Item = (impl Into<String>, String)>,
) {
    let payload = InputBackendLogPayload {
        message: message.into(),
        details: details
            .into_iter()
            .map(|(key, value)| (key.into(), value))
            .collect(),
    };

    let _ = app_handle.emit_to("config", INPUT_BACKEND_LOG_EVENT, payload);
}
