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
        26 => Some("digit-7"),
        28 => Some("digit-8"),
        25 => Some("digit-9"),
        29 => Some("digit-0"),
        27 => Some("minus"),
        24 => Some("equal"),
        48 => Some("tab"),
        12 => Some("q"),
        13 => Some("w"),
        14 => Some("e"),
        15 => Some("r"),
        17 => Some("t"),
        16 => Some("y"),
        32 => Some("u"),
        34 => Some("i"),
        31 => Some("o"),
        35 => Some("p"),
        33 => Some("bracket-left"),
        30 => Some("bracket-right"),
        42 => Some("backslash"),
        0 => Some("a"),
        1 => Some("s"),
        2 => Some("d"),
        3 => Some("f"),
        5 => Some("g"),
        4 => Some("h"),
        38 => Some("j"),
        40 => Some("k"),
        37 => Some("l"),
        41 => Some("semicolon"),
        39 => Some("quote"),
        36 => Some("enter"),
        57 => Some("caps-lock"),
        56 => Some("shift-left"),
        60 => Some("shift-right"),
        6 => Some("z"),
        7 => Some("x"),
        8 => Some("c"),
        9 => Some("v"),
        11 => Some("b"),
        45 => Some("n"),
        46 => Some("m"),
        43 => Some("comma"),
        47 => Some("period"),
        44 => Some("slash"),
        59 => Some("ctrl-left"),
        62 => Some("ctrl-right"),
        55 => Some("alt-left"),
        54 => Some("alt-right"),
        58 => Some("meta-left"),
        61 => Some("meta-right"),
        63 => Some("fn"),
        49 => Some("space"),
        51 => Some("backspace"),
        117 => Some("delete"),
        115 => Some("home"),
        119 => Some("end"),
        116 => Some("page-up"),
        121 => Some("page-down"),
        126 => Some("arrow-up"),
        125 => Some("arrow-down"),
        123 => Some("arrow-left"),
        124 => Some("arrow-right"),
        122 => Some("f1"),
        120 => Some("f2"),
        99 => Some("f3"),
        118 => Some("f4"),
        96 => Some("f5"),
        97 => Some("f6"),
        98 => Some("f7"),
        100 => Some("f8"),
        101 => Some("f9"),
        109 => Some("f10"),
        103 => Some("f11"),
        111 => Some("f12"),
        _ => None,
    }
}

#[cfg_attr(not(any(test, target_os = "windows")), allow(dead_code))]
pub fn key_id_from_windows_event(vk_code: u32, scan_code: u32) -> Option<&'static str> {
    match vk_code {
        0xA3 | 0xA5 | 0x5B | 0x5C => key_id_from_windows_vk(vk_code),
        _ => key_id_from_windows_scancode(scan_code).or_else(|| key_id_from_windows_vk(vk_code)),
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
        0x37 => Some("digit-7"),
        0x38 => Some("digit-8"),
        0x39 => Some("digit-9"),
        0x30 => Some("digit-0"),
        0xBD => Some("minus"),
        0xBB => Some("equal"),
        0x09 => Some("tab"),
        0x41 => Some("a"),
        0x42 => Some("b"),
        0x43 => Some("c"),
        0x44 => Some("d"),
        0x45 => Some("e"),
        0x46 => Some("f"),
        0x47 => Some("g"),
        0x48 => Some("h"),
        0x49 => Some("i"),
        0x4A => Some("j"),
        0x4B => Some("k"),
        0x4C => Some("l"),
        0x4D => Some("m"),
        0x4E => Some("n"),
        0x4F => Some("o"),
        0x50 => Some("p"),
        0x51 => Some("q"),
        0x52 => Some("r"),
        0x53 => Some("s"),
        0x54 => Some("t"),
        0x55 => Some("u"),
        0x56 => Some("v"),
        0x57 => Some("w"),
        0x58 => Some("x"),
        0x59 => Some("y"),
        0x5A => Some("z"),
        0xDB => Some("bracket-left"),
        0xDD => Some("bracket-right"),
        0xDC => Some("backslash"),
        0x14 => Some("caps-lock"),
        0xBA => Some("semicolon"),
        0xDE => Some("quote"),
        0x0D => Some("enter"),
        0xA0 => Some("shift-left"),
        0xA1 => Some("shift-right"),
        0xBC => Some("comma"),
        0xBE => Some("period"),
        0xBF => Some("slash"),
        0xA2 => Some("ctrl-left"),
        0xA3 => Some("ctrl-right"),
        0x5B => Some("meta-left"),
        0x5C => Some("meta-right"),
        0xA4 => Some("alt-left"),
        0xA5 => Some("alt-right"),
        0x20 => Some("space"),
        0x08 => Some("backspace"),
        0x2E => Some("delete"),
        0x2D => Some("insert"),
        0x24 => Some("home"),
        0x23 => Some("end"),
        0x21 => Some("page-up"),
        0x22 => Some("page-down"),
        0x26 => Some("arrow-up"),
        0x28 => Some("arrow-down"),
        0x25 => Some("arrow-left"),
        0x27 => Some("arrow-right"),
        0x70 => Some("f1"),
        0x71 => Some("f2"),
        0x72 => Some("f3"),
        0x73 => Some("f4"),
        0x74 => Some("f5"),
        0x75 => Some("f6"),
        0x76 => Some("f7"),
        0x77 => Some("f8"),
        0x78 => Some("f9"),
        0x79 => Some("f10"),
        0x7A => Some("f11"),
        0x7B => Some("f12"),
        _ => None,
    }
}

#[cfg_attr(not(any(test, target_os = "windows")), allow(dead_code))]
pub fn key_id_from_windows_scancode(scan_code: u32) -> Option<&'static str> {
    match scan_code {
        0x01 => Some("escape"),
        0x29 => Some("backquote"),
        0x02 => Some("digit-1"),
        0x03 => Some("digit-2"),
        0x04 => Some("digit-3"),
        0x05 => Some("digit-4"),
        0x06 => Some("digit-5"),
        0x07 => Some("digit-6"),
        0x08 => Some("digit-7"),
        0x09 => Some("digit-8"),
        0x0A => Some("digit-9"),
        0x0B => Some("digit-0"),
        0x0C => Some("minus"),
        0x0D => Some("equal"),
        0x0F => Some("tab"),
        0x10 => Some("q"),
        0x11 => Some("w"),
        0x12 => Some("e"),
        0x13 => Some("r"),
        0x14 => Some("t"),
        0x15 => Some("y"),
        0x16 => Some("u"),
        0x17 => Some("i"),
        0x18 => Some("o"),
        0x19 => Some("p"),
        0x1A => Some("bracket-left"),
        0x1B => Some("bracket-right"),
        0x2B => Some("backslash"),
        0x1E => Some("a"),
        0x1F => Some("s"),
        0x20 => Some("d"),
        0x21 => Some("f"),
        0x22 => Some("g"),
        0x23 => Some("h"),
        0x24 => Some("j"),
        0x25 => Some("k"),
        0x26 => Some("l"),
        0x27 => Some("semicolon"),
        0x28 => Some("quote"),
        0x1C => Some("enter"),
        0x3A => Some("caps-lock"),
        0x2A => Some("shift-left"),
        0x2C => Some("z"),
        0x2D => Some("x"),
        0x2E => Some("c"),
        0x2F => Some("v"),
        0x30 => Some("b"),
        0x31 => Some("n"),
        0x32 => Some("m"),
        0x33 => Some("comma"),
        0x34 => Some("period"),
        0x35 => Some("slash"),
        0x1D => Some("ctrl-left"),
        0x38 => Some("alt-left"),
        0x39 => Some("space"),
        0x0E => Some("backspace"),
        0x3B => Some("f1"),
        0x3C => Some("f2"),
        0x3D => Some("f3"),
        0x3E => Some("f4"),
        0x3F => Some("f5"),
        0x40 => Some("f6"),
        0x41 => Some("f7"),
        0x42 => Some("f8"),
        0x43 => Some("f9"),
        0x44 => Some("f10"),
        0x57 => Some("f11"),
        0x58 => Some("f12"),
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

// These ids are stable layout handles for native hardware codes that do not
// have a semantic key mapping. They are not key names.
pub fn layout_id_from_macos_keycode(keycode: u16) -> String {
    format!("macos-keycode-{keycode}")
}

#[cfg_attr(not(any(test, target_os = "windows")), allow(dead_code))]
pub fn layout_id_from_windows_codes(vk_code: u32, scan_code: u32) -> String {
    if scan_code > 0 {
        format!("windows-scancode-{scan_code}")
    } else {
        format!("windows-vk-{vk_code}")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        key_id_from_macos_keycode, key_id_from_mouse_button, key_id_from_windows_event,
        key_id_from_windows_scancode, key_id_from_windows_vk, layout_id_from_macos_keycode,
        layout_id_from_windows_codes,
    };

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
        assert_eq!(key_id_from_macos_keycode(26), Some("digit-7"));
        assert_eq!(key_id_from_macos_keycode(28), Some("digit-8"));
        assert_eq!(key_id_from_macos_keycode(25), Some("digit-9"));
        assert_eq!(key_id_from_macos_keycode(29), Some("digit-0"));
        assert_eq!(key_id_from_macos_keycode(48), Some("tab"));
        assert_eq!(key_id_from_macos_keycode(13), Some("w"));
        assert_eq!(key_id_from_macos_keycode(0), Some("a"));
        assert_eq!(key_id_from_macos_keycode(1), Some("s"));
        assert_eq!(key_id_from_macos_keycode(2), Some("d"));
        assert_eq!(key_id_from_macos_keycode(17), Some("t"));
        assert_eq!(key_id_from_macos_keycode(57), Some("caps-lock"));
        assert_eq!(key_id_from_macos_keycode(3), Some("f"));
        assert_eq!(key_id_from_macos_keycode(5), Some("g"));
        assert_eq!(key_id_from_macos_keycode(37), Some("l"));
        assert_eq!(key_id_from_macos_keycode(36), Some("enter"));
        assert_eq!(key_id_from_macos_keycode(51), Some("backspace"));
        assert_eq!(key_id_from_macos_keycode(56), Some("shift-left"));
        assert_eq!(key_id_from_macos_keycode(6), Some("z"));
        assert_eq!(key_id_from_macos_keycode(7), Some("x"));
        assert_eq!(key_id_from_macos_keycode(8), Some("c"));
        assert_eq!(key_id_from_macos_keycode(9), Some("v"));
        assert_eq!(key_id_from_macos_keycode(11), Some("b"));
        assert_eq!(key_id_from_macos_keycode(59), Some("ctrl-left"));
        assert_eq!(key_id_from_macos_keycode(55), Some("alt-left"));
        assert_eq!(key_id_from_macos_keycode(54), Some("alt-right"));
        assert_eq!(key_id_from_macos_keycode(58), Some("meta-left"));
        assert_eq!(key_id_from_macos_keycode(61), Some("meta-right"));
        assert_eq!(key_id_from_macos_keycode(63), Some("fn"));
        assert_eq!(key_id_from_macos_keycode(49), Some("space"));
        assert_eq!(key_id_from_macos_keycode(122), Some("f1"));
        assert_eq!(key_id_from_macos_keycode(111), Some("f12"));
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
        assert_eq!(key_id_from_windows_vk(0x37), Some("digit-7"));
        assert_eq!(key_id_from_windows_vk(0x30), Some("digit-0"));
        assert_eq!(key_id_from_windows_vk(0x09), Some("tab"));
        assert_eq!(key_id_from_windows_vk(0x57), Some("w"));
        assert_eq!(key_id_from_windows_vk(0x41), Some("a"));
        assert_eq!(key_id_from_windows_vk(0x53), Some("s"));
        assert_eq!(key_id_from_windows_vk(0x44), Some("d"));
        assert_eq!(key_id_from_windows_vk(0x54), Some("t"));
        assert_eq!(key_id_from_windows_vk(0x14), Some("caps-lock"));
        assert_eq!(key_id_from_windows_vk(0x46), Some("f"));
        assert_eq!(key_id_from_windows_vk(0x47), Some("g"));
        assert_eq!(key_id_from_windows_vk(0x4C), Some("l"));
        assert_eq!(key_id_from_windows_vk(0x0D), Some("enter"));
        assert_eq!(key_id_from_windows_vk(0x08), Some("backspace"));
        assert_eq!(key_id_from_windows_vk(0xA0), Some("shift-left"));
        assert_eq!(key_id_from_windows_vk(0x5A), Some("z"));
        assert_eq!(key_id_from_windows_vk(0x58), Some("x"));
        assert_eq!(key_id_from_windows_vk(0x43), Some("c"));
        assert_eq!(key_id_from_windows_vk(0x56), Some("v"));
        assert_eq!(key_id_from_windows_vk(0x42), Some("b"));
        assert_eq!(key_id_from_windows_vk(0xA2), Some("ctrl-left"));
        assert_eq!(key_id_from_windows_vk(0x5B), Some("meta-left"));
        assert_eq!(key_id_from_windows_vk(0x5C), Some("meta-right"));
        assert_eq!(key_id_from_windows_vk(0xA4), Some("alt-left"));
        assert_eq!(key_id_from_windows_vk(0xA5), Some("alt-right"));
        assert_eq!(key_id_from_windows_vk(0x20), Some("space"));
        assert_eq!(key_id_from_windows_vk(0x70), Some("f1"));
        assert_eq!(key_id_from_windows_vk(0x7B), Some("f12"));
    }

    #[test]
    fn prefers_windows_virtual_keys_for_right_side_modifiers() {
        assert_eq!(key_id_from_windows_event(0xA5, 0x38), Some("alt-right"));
        assert_eq!(key_id_from_windows_event(0xA4, 0x38), Some("alt-left"));
        assert_eq!(key_id_from_windows_event(0x5C, 0x5C), Some("meta-right"));
    }

    #[test]
    fn maps_windows_scan_codes_to_overlay_ids() {
        assert_eq!(key_id_from_windows_scancode(0x29), Some("backquote"));
        assert_eq!(key_id_from_windows_scancode(0x08), Some("digit-7"));
        assert_eq!(key_id_from_windows_scancode(0x11), Some("w"));
        assert_eq!(key_id_from_windows_scancode(0x26), Some("l"));
        assert_eq!(key_id_from_windows_scancode(0x1C), Some("enter"));
        assert_eq!(key_id_from_windows_scancode(0x2A), Some("shift-left"));
        assert_eq!(key_id_from_windows_scancode(0x39), Some("space"));
        assert_eq!(key_id_from_windows_scancode(0x3B), Some("f1"));
        assert_eq!(key_id_from_windows_scancode(0x58), Some("f12"));
    }

    #[test]
    fn maps_mouse_buttons_to_overlay_ids() {
        assert_eq!(key_id_from_mouse_button(0), Some("mouse-left"));
        assert_eq!(key_id_from_mouse_button(1), Some("mouse-right"));
        assert_eq!(key_id_from_mouse_button(2), None);
    }

    #[test]
    fn leaves_unmapped_native_keys_for_unknown_id_fallback() {
        assert_eq!(key_id_from_macos_keycode(10_000), None);
        assert_eq!(key_id_from_windows_event(0, 10_000), None);
    }

    #[test]
    fn builds_stable_unknown_native_key_ids() {
        assert_eq!(layout_id_from_macos_keycode(123), "macos-keycode-123");
        assert_eq!(layout_id_from_windows_codes(0xFF, 91), "windows-scancode-91");
        assert_eq!(layout_id_from_windows_codes(0xFF, 0), "windows-vk-255");
    }
}
