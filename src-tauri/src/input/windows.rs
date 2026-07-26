use std::{
    ptr::{null, null_mut},
    sync::{
        atomic::{AtomicUsize, Ordering},
        OnceLock,
    },
};
use tauri::AppHandle;
use windows_sys::Win32::{
    Foundation::{GetLastError, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
        KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN,
        WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_RBUTTONDOWN, WM_RBUTTONUP,
        WM_SYSKEYDOWN, WM_SYSKEYUP,
    },
};

use super::{emit_backend_log, emit_input_state, mapping};

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
static KEYBOARD_LOG_COUNT: AtomicUsize = AtomicUsize::new(0);
static MOUSE_LOG_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn start(app_handle: AppHandle) {
    let _ = APP_HANDLE.set(app_handle.clone());
    emit_backend_log(&app_handle, "windows-backend-starting", std::iter::empty::<(&str, String)>());

    std::thread::spawn(move || unsafe {
        let module = GetModuleHandleW(null());
        let keyboard_hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), module, 0);
        let mouse_hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), module, 0);

        if keyboard_hook.is_null() {
            if let Some(app_handle) = APP_HANDLE.get() {
                emit_backend_log(
                    app_handle,
                    "windows-keyboard-hook-failed",
                    [("lastError", GetLastError().to_string())],
                );
            }
            eprintln!("Windows keyboard hook failed to start");
        } else if let Some(app_handle) = APP_HANDLE.get() {
            emit_backend_log(app_handle, "windows-keyboard-hook-started", std::iter::empty::<(&str, String)>());
        }

        if mouse_hook.is_null() {
            if let Some(app_handle) = APP_HANDLE.get() {
                emit_backend_log(
                    app_handle,
                    "windows-mouse-hook-failed",
                    [("lastError", GetLastError().to_string())],
                );
            }
            eprintln!("Windows mouse hook failed to start");
        } else if let Some(app_handle) = APP_HANDLE.get() {
            emit_backend_log(app_handle, "windows-mouse-hook-started", std::iter::empty::<(&str, String)>());
        }

        let mut message = std::mem::zeroed::<MSG>();
        while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    });
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let keyboard = *(lparam as *const KBDLLHOOKSTRUCT);
        let pressed = matches!(wparam as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
        let released = matches!(wparam as u32, WM_KEYUP | WM_SYSKEYUP);

        if pressed || released {
            let key_id = mapping::key_id_from_windows_event(keyboard.vkCode, keyboard.scanCode)
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    mapping::layout_id_from_windows_codes(keyboard.vkCode, keyboard.scanCode)
                });

            if let Some(app_handle) = APP_HANDLE.get() {
                if KEYBOARD_LOG_COUNT.fetch_add(1, Ordering::Relaxed) < 24 {
                    emit_backend_log(
                        app_handle,
                        "windows-keyboard-event",
                        [
                            ("wparam", (wparam as u32).to_string()),
                            ("vkCode", keyboard.vkCode.to_string()),
                            ("scanCode", keyboard.scanCode.to_string()),
                            ("pressed", pressed.to_string()),
                            ("keyId", key_id.clone()),
                        ],
                    );
                }
                emit_input_state(app_handle, key_id, pressed);
            }
        }
    }

    CallNextHookEx(null_mut(), code, wparam, lparam)
}

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let payload = match wparam as u32 {
            WM_LBUTTONDOWN => Some(("mouse-left", true)),
            WM_LBUTTONUP => Some(("mouse-left", false)),
            WM_RBUTTONDOWN => Some(("mouse-right", true)),
            WM_RBUTTONUP => Some(("mouse-right", false)),
            WM_MBUTTONDOWN => Some(("mouse-middle", true)),
            WM_MBUTTONUP => Some(("mouse-middle", false)),
            _ => None,
        };

        if let Some((key_id, pressed)) = payload {
            if let Some(app_handle) = APP_HANDLE.get() {
                if MOUSE_LOG_COUNT.fetch_add(1, Ordering::Relaxed) < 12 {
                    emit_backend_log(
                        app_handle,
                        "windows-mouse-event",
                        [
                            ("wparam", (wparam as u32).to_string()),
                            ("pressed", pressed.to_string()),
                            ("keyId", key_id.to_string()),
                        ],
                    );
                }
                emit_input_state(app_handle, key_id, pressed);
            }
        }
    }

    CallNextHookEx(null_mut(), code, wparam, lparam)
}
