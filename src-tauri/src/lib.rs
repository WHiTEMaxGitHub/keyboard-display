mod commands;
mod debug_log;
mod exporter;
mod input;
mod recording;
mod video_export;

use input::InputStateBridge;
use recording::RecordingManager;
use tauri::RunEvent;

pub fn run() {
    tauri::Builder::default()
        .manage(RecordingManager::new())
        .manage(InputStateBridge::new())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::save_config_file,
            commands::write_debug_log,
            commands::read_config_file,
            commands::load_app_config,
            commands::initialize_app_config,
            commands::app_config_path_string,
            commands::save_app_config,
            commands::default_recording_dir,
            commands::default_export_video_dir,
            commands::start_recording,
            commands::sync_recording_runtime,
            commands::record_input_event,
            commands::add_recording_marker,
            commands::suppress_recording_keys,
            commands::stop_recording,
            commands::inspect_recording_file,
            commands::inspect_recording_export_info,
            commands::list_recording_files,
            commands::create_recording_folder,
            commands::read_recording_metadata,
            commands::save_recording_metadata,
            commands::detect_video_exporter,
            commands::install_app_managed_video_exporter,
            commands::uninstall_app_managed_video_exporter,
            commands::export_overlay_video,
            commands::convert_webm_to_png_mov,
            commands::copy_font_file,
            commands::open_directory
        ])
        .setup(|app| {
            debug_log::init(app.handle());
            input::start_native_input_backend(app.handle().clone());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if matches!(event, RunEvent::Exit) {
                let _ = debug_log::flush();
            }
        });
}
