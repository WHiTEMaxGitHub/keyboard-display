# Platform notes

Keyboard Display depends on desktop-only capabilities that behave differently on
macOS and Windows:

- global keyboard and mouse input capture
- a transparent, click-through POV overlay window
- always-on-top and positioning APIs

This document records the current implementation and the behavior that still
needs real-device verification.

## Current architecture

Input capture starts from `src-tauri/src/input/mod.rs`:

```text
native input backend -> input-state event -> config window / overlay active keys
```

The config window listens for `input-state` events. The POV overlay listens for
the derived active-key set so it can render the current state without owning the
native hook.

The overlay window is created by `src/composables/useOverlayWindow.ts` as a
separate Tauri webview window labeled `pov`. It is configured as:

- undecorated
- transparent with `[0, 0, 0, 0]` background
- hidden from the taskbar
- visible on all workspaces
- sized from the current overlay layout

The window is normally click-through via `setIgnoreCursorEvents(true)`. During
visual adjustment it temporarily disables click-through so the user can drag and
save a custom position.

## macOS

### Input capture

The macOS backend uses a `CGEventTap` at the HID location in listen-only mode.
It subscribes to:

- key down / key up
- modifier flag changes
- left, right, and other mouse button down/up events

Modifier keys are handled through `FlagsChanged` events and raw key codes. This
avoids converting key codes through AppKit or input-source APIs on the hook
thread.

Expected permission requirements:

- Accessibility may be required for global input observation.
- Input Monitoring may be required on newer macOS versions or stricter user
settings.

If the event tap cannot start, the backend currently logs:

```text
macOS native input backend failed to start; check Accessibility/Input Monitoring permissions
```

The app does not yet show a dedicated in-app permission recovery flow.

### Overlay window

The Tauri config enables `macOSPrivateApi` because transparent macOS windows in
Tauri require it. The POV window is transparent, undecorated, and controlled
from the frontend through Tauri window APIs.

Still needs real-device verification:

- transparent background on packaged builds
- click-through behavior after toggling adjustment mode
- always-on-top behavior with full-screen apps and multiple Spaces
- custom-position restoration after monitor/work-area changes

## Windows

### Input capture

The Windows backend uses Raw Input for keyboard events:

- creates a message-only window for `WM_INPUT`
- registers keyboard Raw Input with `RIDEV_INPUTSINK`
- maps virtual-key and scan-code pairs into normalized key ids
- emits fallback ids for unknown scan codes

Mouse buttons use a low-level mouse hook:

- `WH_MOUSE_LL`
- left, right, and middle button down/up events

The message loop and hook live on a dedicated thread.

Still needs real-device verification:

- Raw Input message-window lifetime in dev and release builds
- background keyboard capture while the app is not focused
- mouse hook lifetime after long sessions and suspend/resume
- left/right modifier fidelity on common keyboard layouts

### Overlay window

The POV window is configured as transparent, undecorated, and hidden from the
taskbar. On Windows, `effectiveOverlayAlwaysOnTop` currently forces the overlay
to stay always-on-top even if the profile setting is off, because the overlay
needs to remain visible above the captured game window.

Still needs real-device verification:

- WebView2 transparent background behavior
- click-through behavior through `setIgnoreCursorEvents(true)`
- always-on-top behavior over borderless fullscreen and windowed games
- window sizing and positioning across mixed-DPI monitors

## Unsupported platforms

On platforms other than macOS and Windows, the native input backend logs that it
is unsupported. The app may still render the configuration UI, but global input
capture should not be considered functional.

Frontend keyboard events are useful only for layout editing and browser-level
previews. They are not a fallback for real background capture.

## Fallback behavior when capture is unavailable

If global capture is unavailable because permissions are missing or the backend
cannot start:

- the overlay can still be configured and previewed
- saved profiles remain editable
- recording control UI remains visible
- real background key state may not update
- `.kbdrec` recordings may not contain meaningful input state

Users should grant the required OS permissions or run on a supported desktop
platform before relying on recordings or live POV overlays.

## Verification checklist

Use this checklist when validating a platform build.

macOS:

- Grant Accessibility/Input Monitoring permissions if prompted.
- Confirm normal keys and left/right modifiers update the overlay.
- Confirm mouse buttons update the overlay.
- Confirm no crash occurs after repeated key presses and layout capture.
- Confirm the transparent overlay is click-through outside adjustment mode.
- Confirm adjustment mode can drag, save, and restore a custom position.

Windows:

- Confirm keyboard capture continues while another app is focused.
- Confirm left/right modifiers and mouse buttons update the overlay.
- Confirm Raw Input and mouse hook debug logs show successful startup.
- Confirm the overlay is transparent and click-through.
- Confirm always-on-top behavior over the target game/window mode.
- Confirm positioning on the primary monitor and any mixed-DPI secondary monitor.
