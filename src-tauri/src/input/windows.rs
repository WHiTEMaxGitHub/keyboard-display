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
            RAWINPUTHEADER, RAWKEYBOARD, RAWMOUSE, RIDEV_INPUTSINK, RID_INPUT, RIM_TYPEKEYBOARD,
            RIM_TYPEMOUSE,
        },
        WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetSystemMetrics,
            RegisterClassW, TranslateMessage, MSG, SM_SWAPBUTTON, WM_INPUT, WM_KEYDOWN, WM_KEYUP,
            WM_SYSKEYDOWN, WM_SYSKEYUP, WNDCLASSW,
        },
    },
};

use super::{emit_backend_log, emit_input_state, mapping};

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

const RAW_MOUSE_BUTTON_FLAGS: u16 = 0x003F;

const RAW_INPUT_CLASS_NAME: &[u16] = &[
    'K' as u16, 'e' as u16, 'y' as u16, 'b' as u16, 'o' as u16, 'a' as u16, 'r' as u16, 'd' as u16,
    'D' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16, 'R' as u16,
    'a' as u16, 'w' as u16, 'I' as u16, 'n' as u16, 'p' as u16, 'u' as u16, 't' as u16, 0,
];

pub fn start(app_handle: AppHandle) {
    let _ = APP_HANDLE.set(app_handle.clone());

    std::thread::spawn(move || unsafe {
        let module = GetModuleHandleW(null());
        let raw_input_window = create_raw_input_window(module as HINSTANCE);

        if raw_input_window.is_null() {
            if let Some(app_handle) = APP_HANDLE.get() {
                emit_backend_log(
                    app_handle,
                    "windows-raw-input-window-failed",
                    [("lastError", GetLastError().to_string())],
                );
            }
        }

        let mut message = std::mem::zeroed::<MSG>();
        while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    });
}

unsafe fn create_raw_input_window(instance: HINSTANCE) -> HWND {
    // Raw Input 需要一个 HWND 接收 WM_INPUT。这里创建 message-only window，
    // 避免采集依赖 WebView2 焦点状态。
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

    register_raw_input_devices(hwnd);
    hwnd
}

unsafe fn register_raw_input_devices(hwnd: HWND) {
    // RIDEV_INPUTSINK: receive keyboard/mouse while not foreground.
    // Do not set RIDEV_NOLEGACY (would suppress system legacy mouse messages)
    // or RIDEV_CAPTUREMOUSE.
    let devices = [
        RAWINPUTDEVICE {
            usUsagePage: 0x01,
            usUsage: 0x06,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: hwnd,
        },
        RAWINPUTDEVICE {
            usUsagePage: 0x01,
            usUsage: 0x02,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: hwnd,
        },
    ];
    let ok = RegisterRawInputDevices(
        devices.as_ptr(),
        devices.len() as u32,
        size_of::<RAWINPUTDEVICE>() as u32,
    );
    if ok == 0 {
        if let Some(app_handle) = APP_HANDLE.get() {
            emit_backend_log(
                app_handle,
                "windows-raw-input-register-failed",
                [("lastError", GetLastError().to_string())],
            );
        }
    }
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
    let Some(input) = read_raw_input(raw_input) else {
        return;
    };

    match input.header.dwType {
        RIM_TYPEKEYBOARD => handle_raw_keyboard(input.data.keyboard),
        RIM_TYPEMOUSE => handle_raw_mouse(input.data.mouse),
        _ => {}
    }
}

fn handle_raw_keyboard(keyboard: RAWKEYBOARD) {
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
        // Parse + enqueue only; ingest applies/records off this thread.
        emit_input_state(app_handle, key_id, pressed);
    }
}

fn handle_raw_mouse(mouse: RAWMOUSE) {
    let flags = unsafe { mouse.Anonymous.Anonymous.usButtonFlags };
    if flags & RAW_MOUSE_BUTTON_FLAGS == 0 {
        return;
    }

    // Mouse hold source of truth is Raw Input button flags only. Do not poll
    // GetAsyncKeyState: async UP caused CS2 false LMB drops, and filling DOWN
    // from a stale high bit after a real Raw UP re-lit mouse-left forever.
    // Physical flags vs logical overlay ids: match former WH_MOUSE_LL
    // WM_*BUTTON* behavior, which follows SM_SWAPBUTTON.
    let swap_buttons = unsafe { GetSystemMetrics(SM_SWAPBUTTON) != 0 };
    let Some(app_handle) = APP_HANDLE.get() else {
        return;
    };

    for (key_id, pressed) in mapping::overlay_events_from_raw_mouse_flags(flags, swap_buttons) {
        // Parse + enqueue only; ingest applies/records off this thread.
        emit_input_state(app_handle, key_id, pressed);
    }
}

unsafe fn read_raw_input(raw_input: HRAWINPUT) -> Option<RAWINPUT> {
    let mut input = std::mem::zeroed::<RAWINPUT>();
    let mut size = size_of::<RAWINPUT>() as u32;
    let header_size = size_of::<RAWINPUTHEADER>() as u32;
    let result = GetRawInputData(
        raw_input,
        RID_INPUT,
        (&mut input as *mut RAWINPUT).cast(),
        &mut size,
        header_size,
    );

    if result == u32::MAX || result == 0 {
        None
    } else {
        Some(input)
    }
}

#[cfg(test)]
mod tests {
    use super::super::InputStateBridge;
    use std::{mem::size_of, ptr::null, thread, time::Duration};
    use windows_sys::Win32::{
        Foundation::{HWND, POINT, RECT},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::{
                GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE,
                KEYBDINPUT, KEYEVENTF_KEYUP, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEINPUT,
                VK_LBUTTON, VK_SPACE,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, GetCursorPos, GetWindowRect,
                RegisterClassW, SetCursorPos, SetForegroundWindow, ShowWindow, SW_SHOW, WNDCLASSW,
                WS_POPUP, WS_VISIBLE,
            },
        },
    };

    const ASYNC_KEY_DOWN_MASK: u16 = 0x8000;

    fn keys(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|id| (*id).to_string()).collect()
    }

    fn is_async_key_down(virtual_key: u16) -> bool {
        let state = unsafe { GetAsyncKeyState(i32::from(virtual_key)) };
        (state as u16 & ASYNC_KEY_DOWN_MASK) != 0
    }

    /// GetAsyncKeyState is not a mouse overlay source. Neither missed-DOWN
    /// fill nor async UP is applied — DOWN fill re-lit LMB after a real Raw
    /// UP (stale high bit), and async UP caused CS2 false clears.
    fn mouse_async_reconcile_pressed(_overlay_pressed: bool, _async_pressed: bool) -> Option<bool> {
        None
    }

    fn apply_async_mouse_sample(bridge: &InputStateBridge, key_id: &str, async_pressed: bool) {
        if let Some(pressed) =
            mouse_async_reconcile_pressed(bridge.is_active(key_id), async_pressed)
        {
            bridge.apply_key(key_id, pressed).expect("input state lock");
        }
    }

    #[test]
    fn async_mouse_sample_never_emits_down_or_up() {
        assert_eq!(mouse_async_reconcile_pressed(true, true), None);
        assert_eq!(mouse_async_reconcile_pressed(false, false), None);
        assert_eq!(mouse_async_reconcile_pressed(true, false), None);
        assert_eq!(
            mouse_async_reconcile_pressed(false, true),
            None,
            "stale GetAsyncKeyState down after Raw UP must not fill mouse-left"
        );
    }

    #[test]
    fn stale_async_down_after_raw_up_does_not_stick_mouse_left() {
        let bridge = InputStateBridge::new();
        assert_eq!(
            bridge.apply_key("mouse-left", true).unwrap(),
            keys(&["mouse-left"])
        );
        assert_eq!(bridge.apply_key("mouse-left", false).unwrap(), keys(&[]));
        assert!(!bridge.is_active("mouse-left"));

        // Race: Raw DOWN → Raw UP (overlay empty), then the next ~16ms poll
        // still sees GetAsyncKeyState high bit = 1. A DOWN-only reconciler
        // would emit DOWN and never UP afterwards, sticking mouse-left.
        apply_async_mouse_sample(&bridge, "mouse-left", true);

        assert_eq!(bridge.snapshot(), keys(&[]));
        assert!(
            !bridge.is_active("mouse-left"),
            "overlay must stay clear after Raw UP even if async still reads down"
        );
    }

    #[test]
    fn missed_mouse_left_up_stays_held_when_async_samples_up() {
        let bridge = InputStateBridge::new();
        assert_eq!(
            bridge.apply_key("mouse-left", true).unwrap(),
            keys(&["mouse-left"])
        );
        assert_eq!(
            bridge.apply_key("space", true).unwrap(),
            keys(&["mouse-left", "space"])
        );
        assert_eq!(
            bridge.apply_key("space", false).unwrap(),
            keys(&["mouse-left"])
        );
        assert_eq!(bridge.snapshot(), keys(&["mouse-left"]));

        apply_async_mouse_sample(&bridge, "mouse-left", false);
        assert_eq!(
            bridge.snapshot(),
            keys(&["mouse-left"]),
            "GetAsyncKeyState UP must not clear a held mouse-left"
        );
        assert!(bridge.is_active("mouse-left"));
    }

    #[test]
    fn jump_throw_sendinput_keeps_mouse_left_when_async_samples_up() {
        let mut probe = LiveInputProbe::start().expect("create probe window");
        let bridge = InputStateBridge::new();
        let mut stages = Vec::new();

        probe.send_mouse_left(true);
        thread::sleep(Duration::from_millis(300));
        bridge.apply_key("mouse-left", true).unwrap();
        stages.push(probe.stage("mouse-left down", &bridge));
        assert!(
            is_async_key_down(VK_LBUTTON),
            "GetAsyncKeyState VK_LBUTTON should be down after SendInput: {stages:?}"
        );
        assert_eq!(bridge.snapshot(), keys(&["mouse-left"]));

        probe.send_space(true);
        thread::sleep(Duration::from_millis(20));
        bridge.apply_key("space", true).unwrap();
        stages.push(probe.stage("space down", &bridge));
        assert!(
            is_async_key_down(VK_SPACE),
            "GetAsyncKeyState VK_SPACE should be down after SendInput: {stages:?}"
        );
        assert_eq!(bridge.snapshot(), keys(&["mouse-left", "space"]));

        probe.send_space(false);
        thread::sleep(Duration::from_millis(80));
        bridge.apply_key("space", false).unwrap();
        stages.push(probe.stage("space up", &bridge));
        assert!(
            !is_async_key_down(VK_SPACE),
            "GetAsyncKeyState VK_SPACE should be up after SendInput: {stages:?}"
        );
        assert_eq!(bridge.snapshot(), keys(&["mouse-left"]));

        probe.send_mouse_left(false);
        thread::sleep(Duration::from_millis(20));
        stages.push(probe.stage("mouse-left up (missed overlay apply)", &bridge));
        assert!(
            !is_async_key_down(VK_LBUTTON),
            "GetAsyncKeyState VK_LBUTTON should be up after SendInput: {stages:?}"
        );
        assert_eq!(
            bridge.snapshot(),
            keys(&["mouse-left"]),
            "overlay should still hold mouse-left when Raw Input UP is missed: {stages:?}"
        );

        apply_async_mouse_sample(&bridge, "mouse-left", false);
        stages.push(probe.stage("after GetAsyncKeyState sample (ignored)", &bridge));
        assert!(
            bridge.is_active("mouse-left"),
            "GetAsyncKeyState UP must not clear held mouse-left: {stages:?}"
        );
        assert_eq!(
            bridge.snapshot(),
            keys(&["mouse-left"]),
            "active_keys should keep mouse-left after delayed left-up sample: {stages:?}"
        );

        for stage in &stages {
            eprintln!("{stage}");
        }
    }

    #[test]
    fn sendinput_press_release_does_not_relight_from_stale_async_down() {
        let mut probe = LiveInputProbe::start().expect("create probe window");
        let bridge = InputStateBridge::new();

        probe.send_mouse_left(true);
        thread::sleep(Duration::from_millis(50));
        bridge.apply_key("mouse-left", true).unwrap();
        assert_eq!(bridge.snapshot(), keys(&["mouse-left"]));

        probe.send_mouse_left(false);
        bridge.apply_key("mouse-left", false).unwrap();
        assert_eq!(bridge.snapshot(), keys(&[]));

        // Even if GetAsyncKeyState still reads down right after release,
        // overlay must not be re-lit (the desktop sticky-LMB race).
        apply_async_mouse_sample(&bridge, "mouse-left", true);
        assert!(
            !bridge.is_active("mouse-left"),
            "stale async-down after Raw UP must not stick mouse-left; async={}",
            is_async_key_down(VK_LBUTTON)
        );
        assert_eq!(bridge.snapshot(), keys(&[]));
    }

    const PROBE_CLASS: &[u16] = &[
        'K' as u16, 'D' as u16, 'I' as u16, 'n' as u16, 'p' as u16, 'u' as u16, 't' as u16,
        'P' as u16, 'r' as u16, 'o' as u16, 'b' as u16, 'e' as u16, 0,
    ];

    struct LiveInputProbe {
        hwnd: HWND,
        saved_cursor: POINT,
        mouse_left_down: bool,
        space_down: bool,
    }

    impl LiveInputProbe {
        fn start() -> Result<Self, String> {
            unsafe {
                let module = GetModuleHandleW(null());
                let class = WNDCLASSW {
                    lpfnWndProc: Some(DefWindowProcW),
                    hInstance: module,
                    lpszClassName: PROBE_CLASS.as_ptr(),
                    ..std::mem::zeroed()
                };
                RegisterClassW(&class);

                let hwnd = CreateWindowExW(
                    0,
                    PROBE_CLASS.as_ptr(),
                    PROBE_CLASS.as_ptr(),
                    WS_POPUP | WS_VISIBLE,
                    80,
                    80,
                    96,
                    96,
                    0isize as HWND,
                    std::ptr::null_mut(),
                    module,
                    std::ptr::null_mut(),
                );
                if hwnd.is_null() {
                    return Err("CreateWindowExW failed".into());
                }

                ShowWindow(hwnd, SW_SHOW);
                SetForegroundWindow(hwnd);

                let mut saved_cursor = POINT { x: 0, y: 0 };
                GetCursorPos(&mut saved_cursor);

                let mut rect = RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                GetWindowRect(hwnd, &mut rect);
                SetCursorPos((rect.left + rect.right) / 2, (rect.top + rect.bottom) / 2);
                thread::sleep(Duration::from_millis(50));

                Ok(Self {
                    hwnd,
                    saved_cursor,
                    mouse_left_down: false,
                    space_down: false,
                })
            }
        }

        fn send_mouse_left(&mut self, pressed: bool) {
            let sent = send_mouse_left(pressed);
            assert_eq!(sent, 1, "SendInput mouse-left failed, sent={sent}");
            self.mouse_left_down = pressed;
        }

        fn send_space(&mut self, pressed: bool) {
            let sent = send_key(VK_SPACE, pressed);
            assert_eq!(sent, 1, "SendInput space failed, sent={sent}");
            self.space_down = pressed;
        }

        fn stage(&self, label: &str, bridge: &InputStateBridge) -> String {
            format!(
                "{label}: GetAsyncKeyState LBUTTON={} SPACE={} active_keys={:?}",
                is_async_key_down(VK_LBUTTON),
                is_async_key_down(VK_SPACE),
                bridge.snapshot()
            )
        }
    }

    impl Drop for LiveInputProbe {
        fn drop(&mut self) {
            if self.space_down {
                send_key(VK_SPACE, false);
                self.space_down = false;
            }
            if self.mouse_left_down {
                send_mouse_left(false);
                self.mouse_left_down = false;
            }
            unsafe {
                SetCursorPos(self.saved_cursor.x, self.saved_cursor.y);
                if !self.hwnd.is_null() {
                    DestroyWindow(self.hwnd);
                }
            }
        }
    }

    fn send_mouse_left(pressed: bool) -> u32 {
        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: if pressed {
                        MOUSEEVENTF_LEFTDOWN
                    } else {
                        MOUSEEVENTF_LEFTUP
                    },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        unsafe { SendInput(1, &input, size_of::<INPUT>() as i32) }
    }

    fn send_key(vk: u16, pressed: bool) -> u32 {
        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: if pressed { 0 } else { KEYEVENTF_KEYUP },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        unsafe { SendInput(1, &input, size_of::<INPUT>() as i32) }
    }
}
