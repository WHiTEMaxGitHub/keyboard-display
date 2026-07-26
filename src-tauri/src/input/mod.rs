pub mod mapping;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod unsupported;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

const INPUT_STATE_EVENT: &str = "input-state";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const INPUT_BACKEND_LOG_EVENT: &str = "input-backend-log";

#[derive(Clone, Serialize)]
pub struct InputStatePayload {
    #[serde(rename = "keyId")]
    pub key_id: String,
    pub pressed: bool,
}

#[derive(Clone, Serialize)]
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub struct InputBackendLogPayload {
    pub message: String,
    pub details: std::collections::BTreeMap<String, String>,
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
    let payload = InputStatePayload {
        key_id: key_id.into(),
        pressed,
    };

    for window_label in ["config", "pov"] {
        if let Err(error) = app_handle.emit_to(window_label, INPUT_STATE_EVENT, payload.clone()) {
            eprintln!("failed to emit input state to {window_label}: {error}");
        }
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

    for window_label in ["config", "pov"] {
        let _ = app_handle.emit_to(window_label, INPUT_BACKEND_LOG_EVENT, payload.clone());
    }
}
