mod input;
mod recording;
use recording::RecordingManager;
use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn save_config_file(path: std::path::PathBuf, contents: String) -> Result<(), String> {
    std::fs::write(path, contents).map_err(|error| error.to_string())
}

#[tauri::command]
fn read_config_file(path: std::path::PathBuf) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| error.to_string())
}

#[tauri::command]
fn load_app_config(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let path = app_config_path(&app)?;

    if !path.exists() {
        return Ok(None);
    }

    std::fs::read_to_string(path)
        .map(Some)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn save_app_config(app: tauri::AppHandle, contents: String) -> Result<(), String> {
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
fn default_recording_dir(app: tauri::AppHandle) -> Result<String, String> {
    let path = app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?
        .join("recording-files");

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn start_recording(state: tauri::State<'_, RecordingManager>, fps: u16) -> Result<(), String> {
    state.start(
        fps,
        recording::unix_now_ms()?,
        recording::monotonic_now_ms(),
    )
}

#[tauri::command]
fn record_input_event(
    state: tauri::State<'_, RecordingManager>,
    key_id: String,
    pressed: bool,
) -> Result<(), String> {
    state.record_input(recording::monotonic_now_ms(), key_id, pressed)
}

#[tauri::command]
fn add_recording_marker(
    state: tauri::State<'_, RecordingManager>,
    name: String,
) -> Result<(), String> {
    state.add_marker(recording::monotonic_now_ms(), name)
}

#[tauri::command]
fn suppress_recording_keys(
    state: tauri::State<'_, RecordingManager>,
    key_ids: Vec<String>,
) -> Result<(), String> {
    state.suppress_recent_keys(key_ids)
}

#[tauri::command]
fn stop_recording(
    state: tauri::State<'_, RecordingManager>,
    output_dir: std::path::PathBuf,
) -> Result<recording::StopRecordingResult, String> {
    state.stop(output_dir, recording::unix_now_ms()?)
}

#[tauri::command]
fn inspect_recording_file(
    path: std::path::PathBuf,
) -> Result<recording::RecordingInspection, String> {
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    recording::inspect_kbdrec(&bytes)
}

#[tauri::command]
fn list_recording_files(root: std::path::PathBuf) -> Result<recording::RecordingTreeNode, String> {
    recording::list_recording_files(root)
}

#[tauri::command]
fn read_recording_metadata(
    path: std::path::PathBuf,
) -> Result<recording::RecordingMetadata, String> {
    recording::read_recording_metadata(path)
}

#[tauri::command]
fn save_recording_metadata(
    path: std::path::PathBuf,
    metadata: recording::RecordingMetadata,
) -> Result<recording::RecordingMetadata, String> {
    recording::save_recording_metadata(path, metadata)
}

pub fn run() {
    tauri::Builder::default()
        .manage(RecordingManager::new())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            save_config_file,
            read_config_file,
            load_app_config,
            save_app_config,
            default_recording_dir,
            start_recording,
            record_input_event,
            add_recording_marker,
            suppress_recording_keys,
            stop_recording,
            inspect_recording_file,
            list_recording_files,
            read_recording_metadata,
            save_recording_metadata
        ])
        .setup(|app| {
            input::start_native_input_backend(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
