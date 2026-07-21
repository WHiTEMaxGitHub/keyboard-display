use tauri::Manager;

use crate::recording::{self, RecordingManager};

#[tauri::command]
pub fn save_config_file(path: std::path::PathBuf, contents: String) -> Result<(), String> {
    std::fs::write(path, contents).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn read_config_file(path: std::path::PathBuf) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn load_app_config(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let path = app_config_path(&app)?;

    if !path.exists() {
        return Ok(None);
    }

    std::fs::read_to_string(path)
        .map(Some)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_app_config(app: tauri::AppHandle, contents: String) -> Result<(), String> {
    let path = app_config_path(&app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    std::fs::write(path, contents).map_err(|error| error.to_string())
}

fn app_config_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?
        .join("app-config.json"))
}

#[tauri::command]
pub fn default_recording_dir(app: tauri::AppHandle) -> Result<String, String> {
    let path = app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?
        .join("recording-files");

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn start_recording(state: tauri::State<'_, RecordingManager>, fps: u16) -> Result<(), String> {
    state.start(
        fps,
        recording::unix_now_ms()?,
        recording::monotonic_now_ms(),
    )
}

#[tauri::command]
pub fn record_input_event(
    state: tauri::State<'_, RecordingManager>,
    key_id: String,
    pressed: bool,
) -> Result<(), String> {
    state.record_input(recording::monotonic_now_ms(), key_id, pressed)
}

#[tauri::command]
pub fn add_recording_marker(
    state: tauri::State<'_, RecordingManager>,
    name: String,
) -> Result<(), String> {
    state.add_marker(recording::monotonic_now_ms(), name)
}

#[tauri::command]
pub fn suppress_recording_keys(
    state: tauri::State<'_, RecordingManager>,
    key_ids: Vec<String>,
) -> Result<(), String> {
    state.suppress_recent_keys(key_ids)
}

#[tauri::command]
pub fn stop_recording(
    state: tauri::State<'_, RecordingManager>,
    output_dir: std::path::PathBuf,
    filename_template: String,
    profile_name: String,
    fps: u16,
) -> Result<recording::StopRecordingResult, String> {
    state.stop_with_filename_template(
        output_dir,
        recording::unix_now_ms()?,
        &filename_template,
        &profile_name,
        fps,
    )
}

#[tauri::command]
pub fn inspect_recording_file(
    path: std::path::PathBuf,
) -> Result<recording::RecordingInspection, String> {
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    recording::inspect_kbdrec(&bytes)
}

#[tauri::command]
pub fn list_recording_files(
    root: std::path::PathBuf,
) -> Result<recording::RecordingTreeNode, String> {
    recording::list_recording_files(root)
}

#[tauri::command]
pub fn create_recording_folder(
    root: std::path::PathBuf,
    folder_name: String,
) -> Result<recording::RecordingTreeNode, String> {
    recording::create_recording_folder(root, folder_name)
}

#[tauri::command]
pub fn read_recording_metadata(
    path: std::path::PathBuf,
) -> Result<recording::RecordingMetadata, String> {
    recording::read_recording_metadata(path)
}

#[tauri::command]
pub fn save_recording_metadata(
    path: std::path::PathBuf,
    metadata: recording::RecordingMetadata,
) -> Result<recording::RecordingMetadata, String> {
    recording::save_recording_metadata(path, metadata)
}
