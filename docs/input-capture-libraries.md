# Input capture library evaluation

Keyboard Display needs full-state global input capture for a live overlay and
for `.kbdrec` recording. This is broader than registering a few global hotkeys:

- detect key down and key up, not only shortcuts
- preserve left/right modifier identity
- capture mouse button state
- work while a game or another desktop app is focused
- provide stable raw/fallback ids for unusual keys
- avoid high-frequency logging or allocation-heavy hot paths

## Current baseline

The current baseline remains the project-owned native backend:

- macOS: `CGEventTap` in listen-only HID mode
- Windows keyboard: Raw Input through a message-only `WM_INPUT` window
- Windows mouse: `WH_MOUSE_LL`
- shared normalization: `src-tauri/src/input/mapping.rs`

This keeps the code close to the OS event data that the overlay and recorder
actually need.

## Candidate summary

| Candidate | Fit | Notes |
| --- | --- | --- |
| `rdev` | Do not adopt now | It matches the general "global input events" shape, but this project already hit a macOS crash path through `Keyboard::string_from_code` / input-source APIs. Re-introducing it would need a dedicated crash-regression pass. |
| `tauri-plugin-global-shortcut` / `global-hotkey` | Not sufficient | Good for app commands and shortcut registration. It does not provide the continuous full-state key/mouse stream needed by the overlay and recorder. |
| `device_query` / polling-style crates | Not sufficient | Polling can inspect current state, but it is weaker for exact down/up transitions, high-rate recording, and background game workflows. It also does not solve platform-specific permission and mapping details. |
| `raw-input` crate | Possible experiment only | Its stated direction overlaps with this app's Windows Raw Input and macOS CGEventTap needs. It still needs a proof-of-concept against left/right modifiers, mouse buttons, background capture, and macOS permission behavior before it can replace local backends. |
| Linux-oriented event crates | Out of current scope | The app is currently desktop-focused on macOS and Windows. Linux/Wayland/X11 capture needs a separate product decision and permission model. |

## Decision

Do not replace the current backends in this pass.

The current code has already encoded project-specific requirements that generic
libraries usually flatten away:

- Windows Raw Input is required because low-level keyboard hooks did not behave
  reliably when WebView2 had focus.
- macOS must avoid input-source string conversion on the event-tap thread.
- The frontend layout editor needs stable fallback ids for unknown physical
  keys.
- Recording control hotkeys must be filtered from `.kbdrec` input state without
  losing other simultaneous keys.

Keep the local native backends as the baseline until a candidate can prove it
meets those requirements.

## Future proof-of-concept checklist

Before adopting any library, create a throwaway branch and verify:

- `W`, `A`, `S`, `D`, space, tab, escape, and function keys
- left/right shift, control, option/alt, command/windows
- left, right, and middle mouse buttons
- unknown-key fallback ids
- capture while the app is unfocused and a game/window is focused
- macOS Accessibility/Input Monitoring behavior
- Windows WebView2 focused-window behavior
- event ordering under key chords
- no crash path equivalent to the prior `rdev` issue
- no high-frequency logging in hot paths

Only after the proof-of-concept passes should the library be considered for a
real backend replacement.
