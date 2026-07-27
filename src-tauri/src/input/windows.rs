use std::{
    mem::size_of,
    ptr::{null, null_mut},
    sync::OnceLock,
};
use tauri::AppHandle;
use windows_sys::Win32::{
    Foundation::{GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        Input::{
            GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE,
            RAWINPUTHEADER, RID_INPUT, RIDEV_INPUTSINK, RIM_TYPEKEYBOARD,
        },
        WindowsAndMessaging::{
            CallNextHookEx, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW,
            RegisterClassW, SetWindowsHookExW, TranslateMessage, MSG, WNDCLASSW, WH_MOUSE_LL,
            WM_INPUT, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN,
            WM_MBUTTONUP, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
        },
    },
};

use super::{emit_backend_log, emit_input_state, mapping};

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

const RAW_INPUT_CLASS_NAME: &[u16] = &[
    'K' as u16, 'e' as u16, 'y' as u16, 'b' as u16, 'o' as u16, 'a' as u16, 'r' as u16,
    'd' as u16, 'D' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16,
    'y' as u16, 'R' as u16, 'a' as u16, 'w' as u16, 'I' as u16, 'n' as u16, 'p' as u16,
    'u' as u16, 't' as u16, 0,
];

pub fn start(app_handle: AppHandle) {
    let _ = APP_HANDLE.set(app_handle.clone());
    emit_backend_log(&app_handle, "windows-backend-starting", std::iter::empty::<(&str, String)>());

    std::thread::spawn(move || unsafe {
        let module = GetModuleHandleW(null());
        let raw_input_window = create_raw_input_window(module as HINSTANCE);
        let mouse_hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), module, 0);

        if raw_input_window.is_null() {
            if let Some(app_handle) = APP_HANDLE.get() {
                emit_backend_log(
                    app_handle,
                    "windows-raw-input-window-failed",
                    [("lastError", GetLastError().to_string())],
                );
            }
        } else if let Some(app_handle) = APP_HANDLE.get() {
            emit_backend_log(app_handle, "windows-raw-input-window-started", std::iter::empty::<(&str, String)>());
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

unsafe fn create_raw_input_window(instance: HINSTANCE) -> HWND {
    let class = WNDCLASSW {
        lpfnWndProc: Some(raw_input_window_proc),
        hInstance: instance,
        lpszClassName: RAW_INPUT_CLASS_NAME.as_ptr(),
        ..std::mem::zeroed()
    };
    RegisterClassW(&class);

    let hwnd = CreateWindowExW(
        0,
        RAW_INPUT_CLASS_NAME.as_ptr(),
        RAW_INPUT_CLASS_NAME.as_ptr(),
        0,
        0,
        0,
        0,
        0,
        -3isize as HWND,
        null_mut(),
        instance,
        null_mut(),
    );

    if hwnd.is_null() {
        return hwnd;
    }

    let device = RAWINPUTDEVICE {
        usUsagePage: 0x01,
        usUsage: 0x06,
        dwFlags: RIDEV_INPUTSINK,
        hwndTarget: hwnd,
    };
    let ok = RegisterRawInputDevices(&device, 1, size_of::<RAWINPUTDEVICE>() as u32);
    if ok == 0 {
        if let Some(app_handle) = APP_HANDLE.get() {
            emit_backend_log(
                app_handle,
                "windows-raw-input-register-failed",
                [("lastError", GetLastError().to_string())],
            );
        }
    } else if let Some(app_handle) = APP_HANDLE.get() {
        emit_backend_log(app_handle, "windows-raw-input-registered", std::iter::empty::<(&str, String)>());
    }

    hwnd
}

unsafe extern "system" fn raw_input_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_INPUT {
        handle_raw_input(lparam as HRAWINPUT);
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}

unsafe fn handle_raw_input(raw_input: HRAWINPUT) {
    let mut size = 0_u32;
    let header_size = size_of::<RAWINPUTHEADER>() as u32;
    if GetRawInputData(raw_input, RID_INPUT, null_mut(), &mut size, header_size) == u32::MAX {
        return;
    }

    let mut buffer = vec![0_u8; size as usize];
    if GetRawInputData(
        raw_input,
        RID_INPUT,
        buffer.as_mut_ptr().cast(),
        &mut size,
        header_size,
    ) == u32::MAX
    {
        return;
    }

    let input = &*(buffer.as_ptr() as *const RAWINPUT);
    if input.header.dwType != RIM_TYPEKEYBOARD {
        return;
    }

    let keyboard = input.data.keyboard;
    let pressed = matches!(keyboard.Message, WM_KEYDOWN | WM_SYSKEYDOWN);
    let released = matches!(keyboard.Message, WM_KEYUP | WM_SYSKEYUP);
    if !pressed && !released {
        return;
    }

    let vk_code = u32::from(keyboard.VKey);
    let scan_code = u32::from(keyboard.MakeCode);
    let key_id = mapping::key_id_from_windows_event(vk_code, scan_code)
        .map(str::to_owned)
        .unwrap_or_else(|| mapping::layout_id_from_windows_codes(vk_code, scan_code));

    if let Some(app_handle) = APP_HANDLE.get() {
        emit_input_state(app_handle, key_id, pressed);
    }
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
                emit_input_state(app_handle, key_id, pressed);
            }
        }
    }

    CallNextHookEx(null_mut(), code, wparam, lparam)
}
