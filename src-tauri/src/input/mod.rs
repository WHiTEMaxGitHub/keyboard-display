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

#[derive(Clone, Serialize)]
pub struct InputStatePayload {
    #[serde(rename = "keyId")]
    pub key_id: String,
    pub pressed: bool,
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

    if let Err(error) = app_handle.emit(INPUT_STATE_EVENT, payload) {
        eprintln!("failed to emit input state: {error}");
    }
}
