mod input;
mod recording;
use recording::RecordingManager;

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
fn start_recording(state: tauri::State<'_, RecordingManager>, fps: u16) -> Result<(), String> {
    state.start(fps, recording::now_ms()?)
}

#[tauri::command]
fn record_input_event(
    state: tauri::State<'_, RecordingManager>,
    key_id: String,
    pressed: bool,
) -> Result<(), String> {
    state.record_input(recording::now_ms()?, key_id, pressed)
}

#[tauri::command]
fn stop_recording(
    state: tauri::State<'_, RecordingManager>,
    output_dir: std::path::PathBuf,
) -> Result<recording::StopRecordingResult, String> {
    state.stop(output_dir, recording::now_ms()?)
}

pub fn run() {
    tauri::Builder::default()
        .manage(RecordingManager::new())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            save_config_file,
            start_recording,
            record_input_event,
            stop_recording
        ])
        .setup(|app| {
            input::start_native_input_backend(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
