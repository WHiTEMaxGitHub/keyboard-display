use tauri::AppHandle;

pub fn start(_app_handle: AppHandle) {
    crate::debug_log::warn("input-backend", "native input backend is unsupported");
    eprintln!("native input backend is unsupported on this platform");
}
