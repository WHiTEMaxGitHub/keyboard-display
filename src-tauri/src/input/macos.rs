use core_foundation::runloop::CFRunLoop;
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, CallbackResult, EventField,
};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
use tauri::AppHandle;

use crate::debug_log;

use super::{emit_input_state, mapping};

pub fn start(app_handle: AppHandle) {
    std::thread::spawn(move || {
        let modifier_state = Arc::new(Mutex::new(HashSet::new()));
        let callback_modifier_state = Arc::clone(&modifier_state);

        let result = CGEventTap::with_enabled(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            vec![
                CGEventType::KeyDown,
                CGEventType::KeyUp,
                CGEventType::FlagsChanged,
                CGEventType::LeftMouseDown,
                CGEventType::LeftMouseUp,
                CGEventType::RightMouseDown,
                CGEventType::RightMouseUp,
                CGEventType::OtherMouseDown,
                CGEventType::OtherMouseUp,
            ],
            move |_proxy, event_type, event| {
                handle_event(&app_handle, &callback_modifier_state, event_type, event);
                CallbackResult::Keep
            },
            CFRunLoop::run_current,
        );

        if result.is_err() {
            debug_log::warn(
                "input-backend",
                "macos-event-tap-failed check Accessibility/Input Monitoring permissions",
            );
            eprintln!(
                "macOS native input backend failed to start; check Accessibility/Input Monitoring permissions"
            );
        }
    });
}

fn handle_event(
    app_handle: &AppHandle,
    modifier_state: &Mutex<HashSet<u16>>,
    event_type: CGEventType,
    event: &CGEvent,
) {
    match event_type {
        CGEventType::KeyDown | CGEventType::KeyUp => {
            let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;
            if is_macos_modifier_keycode(keycode) {
                return;
            }

            let key_id = mapping::key_id_from_macos_keycode(keycode)
                .map(str::to_owned)
                .unwrap_or_else(|| mapping::layout_id_from_macos_keycode(keycode));

            emit_input_state(
                app_handle,
                key_id,
                matches!(event_type, CGEventType::KeyDown),
            );
        }
        CGEventType::FlagsChanged => {
            let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;
            let key_id = mapping::key_id_from_macos_keycode(keycode)
                .map(str::to_owned)
                .unwrap_or_else(|| mapping::layout_id_from_macos_keycode(keycode));

            if key_id == "caps-lock" {
                emit_pulse(app_handle, key_id);
                return;
            }

            let flags = event.get_flags();
            let pressed = is_modifier_pressed(keycode, flags);
            update_modifier_state(modifier_state, keycode, pressed);

            emit_input_state(app_handle, key_id, pressed);
        }
        CGEventType::LeftMouseDown | CGEventType::LeftMouseUp => {
            if let Some(key_id) = mapping::key_id_from_mouse_button(0) {
                emit_input_state(
                    app_handle,
                    key_id,
                    matches!(event_type, CGEventType::LeftMouseDown),
                );
            }
        }
        CGEventType::RightMouseDown | CGEventType::RightMouseUp => {
            if let Some(key_id) = mapping::key_id_from_mouse_button(1) {
                emit_input_state(
                    app_handle,
                    key_id,
                    matches!(event_type, CGEventType::RightMouseDown),
                );
            }
        }
        CGEventType::OtherMouseDown | CGEventType::OtherMouseUp => {
            let button =
                event.get_integer_value_field(EventField::MOUSE_EVENT_BUTTON_NUMBER) as u16;
            if let Some(key_id) = mapping::key_id_from_mouse_button(button) {
                emit_input_state(
                    app_handle,
                    key_id,
                    matches!(event_type, CGEventType::OtherMouseDown),
                );
            }
        }
        _ => {}
    }
}

fn is_macos_modifier_keycode(keycode: u16) -> bool {
    matches!(keycode, 56 | 60 | 59 | 62 | 55 | 54 | 58 | 61 | 57 | 63)
}

fn is_modifier_pressed(keycode: u16, flags: CGEventFlags) -> bool {
    match keycode {
        56 | 60 => flags.contains(CGEventFlags::CGEventFlagShift),
        59 | 62 => flags.contains(CGEventFlags::CGEventFlagControl),
        55 | 54 => flags.contains(CGEventFlags::CGEventFlagCommand),
        58 | 61 => flags.contains(CGEventFlags::CGEventFlagAlternate),
        63 => flags.contains(CGEventFlags::CGEventFlagSecondaryFn),
        _ => false,
    }
}

fn update_modifier_state(modifier_state: &Mutex<HashSet<u16>>, keycode: u16, pressed: bool) {
    let Ok(mut state) = modifier_state.lock() else {
        return;
    };

    if pressed {
        state.insert(keycode);
    } else {
        state.remove(&keycode);
    }
}

fn emit_pulse(app_handle: &AppHandle, key_id: String) {
    emit_input_state(app_handle, key_id.clone(), true);

    let app_handle = app_handle.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(80));
        emit_input_state(&app_handle, key_id, false);
    });
}
