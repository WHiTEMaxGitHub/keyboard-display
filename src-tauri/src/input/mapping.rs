pub fn key_id_from_macos_keycode(keycode: u16) -> Option<&'static str> {
    match keycode {
        53 => Some("escape"),
        50 => Some("backquote"),
        18 => Some("digit-1"),
        19 => Some("digit-2"),
        20 => Some("digit-3"),
        21 => Some("digit-4"),
        23 => Some("digit-5"),
        22 => Some("digit-6"),
        48 => Some("tab"),
        13 => Some("w"),
        0 => Some("a"),
        1 => Some("s"),
        2 => Some("d"),
        17 => Some("t"),
        57 => Some("caps-lock"),
        3 => Some("f"),
        5 => Some("g"),
        56 => Some("shift-left"),
        60 => Some("shift-right"),
        6 => Some("z"),
        7 => Some("x"),
        8 => Some("c"),
        9 => Some("v"),
        11 => Some("b"),
        59 => Some("ctrl-left"),
        62 => Some("ctrl-right"),
        55 => Some("meta-left"),
        58 => Some("alt-left"),
        49 => Some("space"),
        15 => Some("r"),
        12 => Some("q"),
        14 => Some("e"),
        _ => None,
    }
}

#[cfg_attr(not(any(test, target_os = "windows")), allow(dead_code))]
pub fn key_id_from_windows_vk(vk_code: u32) -> Option<&'static str> {
    match vk_code {
        0x1B => Some("escape"),
        0xC0 => Some("backquote"),
        0x31 => Some("digit-1"),
        0x32 => Some("digit-2"),
        0x33 => Some("digit-3"),
        0x34 => Some("digit-4"),
        0x35 => Some("digit-5"),
        0x36 => Some("digit-6"),
        0x09 => Some("tab"),
        0x57 => Some("w"),
        0x41 => Some("a"),
        0x53 => Some("s"),
        0x44 => Some("d"),
        0x54 => Some("t"),
        0x14 => Some("caps-lock"),
        0x46 => Some("f"),
        0x47 => Some("g"),
        0xA0 => Some("shift-left"),
        0xA1 => Some("shift-right"),
        0x5A => Some("z"),
        0x58 => Some("x"),
        0x43 => Some("c"),
        0x56 => Some("v"),
        0x42 => Some("b"),
        0xA2 => Some("ctrl-left"),
        0xA3 => Some("ctrl-right"),
        0x5B => Some("meta-left"),
        0xA4 => Some("alt-left"),
        0x20 => Some("space"),
        0x52 => Some("r"),
        0x51 => Some("q"),
        0x45 => Some("e"),
        _ => None,
    }
}

pub fn key_id_from_mouse_button(button: u16) -> Option<&'static str> {
    match button {
        0 => Some("mouse-left"),
        1 => Some("mouse-right"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{key_id_from_macos_keycode, key_id_from_mouse_button, key_id_from_windows_vk};

    #[test]
    fn maps_macos_ansi_keycodes_to_overlay_ids() {
        assert_eq!(key_id_from_macos_keycode(53), Some("escape"));
        assert_eq!(key_id_from_macos_keycode(50), Some("backquote"));
        assert_eq!(key_id_from_macos_keycode(18), Some("digit-1"));
        assert_eq!(key_id_from_macos_keycode(19), Some("digit-2"));
        assert_eq!(key_id_from_macos_keycode(20), Some("digit-3"));
        assert_eq!(key_id_from_macos_keycode(21), Some("digit-4"));
        assert_eq!(key_id_from_macos_keycode(23), Some("digit-5"));
        assert_eq!(key_id_from_macos_keycode(22), Some("digit-6"));
        assert_eq!(key_id_from_macos_keycode(48), Some("tab"));
        assert_eq!(key_id_from_macos_keycode(13), Some("w"));
        assert_eq!(key_id_from_macos_keycode(0), Some("a"));
        assert_eq!(key_id_from_macos_keycode(1), Some("s"));
        assert_eq!(key_id_from_macos_keycode(2), Some("d"));
        assert_eq!(key_id_from_macos_keycode(17), Some("t"));
        assert_eq!(key_id_from_macos_keycode(57), Some("caps-lock"));
        assert_eq!(key_id_from_macos_keycode(3), Some("f"));
        assert_eq!(key_id_from_macos_keycode(5), Some("g"));
        assert_eq!(key_id_from_macos_keycode(56), Some("shift-left"));
        assert_eq!(key_id_from_macos_keycode(6), Some("z"));
        assert_eq!(key_id_from_macos_keycode(7), Some("x"));
        assert_eq!(key_id_from_macos_keycode(8), Some("c"));
        assert_eq!(key_id_from_macos_keycode(9), Some("v"));
        assert_eq!(key_id_from_macos_keycode(11), Some("b"));
        assert_eq!(key_id_from_macos_keycode(59), Some("ctrl-left"));
        assert_eq!(key_id_from_macos_keycode(55), Some("meta-left"));
        assert_eq!(key_id_from_macos_keycode(58), Some("alt-left"));
        assert_eq!(key_id_from_macos_keycode(49), Some("space"));
    }

    #[test]
    fn maps_windows_virtual_keys_to_overlay_ids() {
        assert_eq!(key_id_from_windows_vk(0x1B), Some("escape"));
        assert_eq!(key_id_from_windows_vk(0xC0), Some("backquote"));
        assert_eq!(key_id_from_windows_vk(0x31), Some("digit-1"));
        assert_eq!(key_id_from_windows_vk(0x32), Some("digit-2"));
        assert_eq!(key_id_from_windows_vk(0x33), Some("digit-3"));
        assert_eq!(key_id_from_windows_vk(0x34), Some("digit-4"));
        assert_eq!(key_id_from_windows_vk(0x35), Some("digit-5"));
        assert_eq!(key_id_from_windows_vk(0x36), Some("digit-6"));
        assert_eq!(key_id_from_windows_vk(0x09), Some("tab"));
        assert_eq!(key_id_from_windows_vk(0x57), Some("w"));
        assert_eq!(key_id_from_windows_vk(0x41), Some("a"));
        assert_eq!(key_id_from_windows_vk(0x53), Some("s"));
        assert_eq!(key_id_from_windows_vk(0x44), Some("d"));
        assert_eq!(key_id_from_windows_vk(0x54), Some("t"));
        assert_eq!(key_id_from_windows_vk(0x14), Some("caps-lock"));
        assert_eq!(key_id_from_windows_vk(0x46), Some("f"));
        assert_eq!(key_id_from_windows_vk(0x47), Some("g"));
        assert_eq!(key_id_from_windows_vk(0xA0), Some("shift-left"));
        assert_eq!(key_id_from_windows_vk(0x5A), Some("z"));
        assert_eq!(key_id_from_windows_vk(0x58), Some("x"));
        assert_eq!(key_id_from_windows_vk(0x43), Some("c"));
        assert_eq!(key_id_from_windows_vk(0x56), Some("v"));
        assert_eq!(key_id_from_windows_vk(0x42), Some("b"));
        assert_eq!(key_id_from_windows_vk(0xA2), Some("ctrl-left"));
        assert_eq!(key_id_from_windows_vk(0x5B), Some("meta-left"));
        assert_eq!(key_id_from_windows_vk(0xA4), Some("alt-left"));
        assert_eq!(key_id_from_windows_vk(0x20), Some("space"));
    }

    #[test]
    fn maps_mouse_buttons_to_overlay_ids() {
        assert_eq!(key_id_from_mouse_button(0), Some("mouse-left"));
        assert_eq!(key_id_from_mouse_button(1), Some("mouse-right"));
        assert_eq!(key_id_from_mouse_button(2), None);
    }
}
