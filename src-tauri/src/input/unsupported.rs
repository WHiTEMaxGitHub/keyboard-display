use tauri::AppHandle;

pub fn start(_app_handle: AppHandle) {
    eprintln!("native input backend is unsupported on this platform");
}
