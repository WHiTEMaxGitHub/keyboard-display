use std::{
    ptr::{null, null_mut},
    sync::OnceLock,
};
use tauri::AppHandle;
use windows_sys::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
        KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN,
        WM_LBUTTONUP, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    },
};

use super::{emit_input_state, mapping};

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub fn start(app_handle: AppHandle) {
    let _ = APP_HANDLE.set(app_handle);

    std::thread::spawn(move || unsafe {
        let module = GetModuleHandleW(null());
        let keyboard_hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), module, 0);
        let mouse_hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), module, 0);

        if keyboard_hook.is_null() {
            eprintln!("Windows keyboard hook failed to start");
        }

        if mouse_hook.is_null() {
            eprintln!("Windows mouse hook failed to start");
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
            _ => None,
        };

        if let Some((key_id, pressed)) = payload {
            if let Some(app_handle) = APP_HANDLE.get() {
                emit_input_state(app_handle, key_id, pressed);
            }
        }
    }

    CallNextHookEx(null_mut(), code, wparam, lparam)
}
