mod commands;
mod exporter;
mod input;
mod recording;

use recording::RecordingManager;

pub fn run() {
    tauri::Builder::default()
        .manage(RecordingManager::new())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::save_config_file,
            commands::read_config_file,
            commands::load_app_config,
            commands::save_app_config,
            commands::default_recording_dir,
            commands::start_recording,
            commands::record_input_event,
            commands::add_recording_marker,
            commands::suppress_recording_keys,
            commands::stop_recording,
            commands::inspect_recording_file,
            commands::list_recording_files,
            commands::create_recording_folder,
            commands::read_recording_metadata,
            commands::save_recording_metadata,
            commands::detect_video_exporter,
            commands::install_app_managed_video_exporter,
            commands::uninstall_app_managed_video_exporter
        ])
        .setup(|app| {
            input::start_native_input_backend(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
