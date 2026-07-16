# TODO

## Overlay color configuration

- [x] Split text color into separate idle and active colors.
  - Current model:
    - `textColor`
  - Target model:
    - `idleTextColor`
    - `activeTextColor`
  - Reason: idle keys and pressed keys may use very different backgrounds, so a
    single text color cannot always keep good contrast.

- [ ] Replace native `<input type="color">` with an in-app color picker.
  - Native color input is useful for the first version, but its UI and behavior
    may differ between Windows WebView2 and macOS WKWebView.
  - A custom picker should provide consistent behavior across platforms.
  - Desired features:
    - HEX input
    - color preview
    - hue control
    - saturation/value control
    - optional alpha slider where useful
    - preset colors
    - recent colors

- [ ] Keep opacity separate from color when it affects rendering semantics.
  - Overlay opacity controls the whole POV overlay.
  - Backplate opacity controls only the backplate.
  - Key color alpha can be added later if the custom picker supports it.

## Internationalization

- [ ] Add multi-language support for the configuration UI and user-facing text.
  - Prefer one file per language, for example:
    - `locales/en.json`
    - `locales/zh-CN.json`
    - `locales/ja.json`
  - Keep translation files self-contained so non-developer translators can work
    on one language without touching source code.
  - Use stable translation keys instead of source text as keys.
  - Keep placeholders explicit and named, for example:
    - `"profile.loaded": "Loaded {name}"`
  - Avoid splitting one visible sentence across multiple translation keys.
  - Add a fallback language, likely English.
  - Add a simple missing-key check before release.

## Recording sync

- [ ] Implement `.kbdrec` binary v1 as an event stream.
  - Record input transitions per capture frame, not every rendered frame state.
  - Use a simple event-stream format first, not Huffman.
  - Format shape:
    - Header
    - Key table
    - Event stream
    - Marker stream
  - Header fields:
    - magic: `"KBDR"`
    - version: `1`
    - flags
    - fps
    - key_count
    - event_count
    - marker_count
  - Key table:
    - stores only real recordable keys
    - excludes virtual layout-only keys such as `void`
    - maps profile key id to compact numeric key id
  - Event stream:
    - each event stores changes that happened at one capture frame
    - use `frame_delta varint` instead of absolute frame numbers
    - event payload:
      - `down_count varint + down_key_ids`
      - `up_count varint + up_key_ids`
    - playback reconstructs pressed state by applying `down` then `up`
  - Marker stream:
    - `marker_count varint`
    - each marker: `t_ms varint + name_len varint + utf8 name`
  - Future optional sections:
    - periodic state checkpoints for fast seeking
    - RLE state bitset stream for video export caches
  - Keep the current JSON event stream as a debug / test format.
  - Consider Huffman only as a future v2 after collecting real samples.

- [ ] Use monotonic timestamps for input recordings.
  - Store event time as `t = now_monotonic - record_start_monotonic`.
  - Do not rely on wall-clock time for frame/event alignment.

- [ ] Add sync markers to the recording format.
  - Example event:
    - `{ "t": 18342, "type": "marker", "name": "sync" }`
  - Markers should be visible in the UI timeline/list.

- [ ] Add a manual sync marker action.
  - Button: `Add sync marker`.
  - Hotkey candidate: `F8`.
  - The marker should be written into the `.kbdrec` event stream.

- [ ] Add visible sync feedback.
  - Flash the POV overlay when a sync marker is inserted.
  - Optionally play a short beep.
  - This gives the video timeline a visible/audio cue for alignment.

- [ ] Support export offset.
  - Export overlay video with an `offsetMs` option.
  - Use this to align `.kbdrec` playback with external game footage.

- [ ] Document two recording workflows.
  - Realtime OBS capture: overlay is recorded directly, no later alignment
    needed.
  - Post-production export: `.kbdrec` is rendered later and aligned with sync
    markers.

## App configuration

- [x] Implement app-level configuration persistence.
  - App config is separate from POV profile config.
  - Current app config stores the active profile state directly so unsaved
    profile edits survive app restart.
  - Persisted recording state currently includes:
    - recording output directory
    - recording hotkeys
    - silent recording

- [x] Load app config on startup.
  - Restores the active profile state.
  - Restores the last recording output directory.
  - Restores recording hotkeys and silent recording.

- [x] Save app config when user changes app-level state.
  - Current profile state.
  - Profile source path.
  - Recording output directory.
  - Recording hotkeys.
  - Silent recording.

- [ ] Add recent profile list.
  - Keep only real user-selected profile paths.
  - Use it for quick switching in the config UI.

- [ ] Add UI language setting after i18n exists.
  - Store only the selected language code in app config.

## Cross-platform gaps

- [ ] Implement the Windows native input backend.
  - Current state:
    - macOS backend has an initial `CGEventTap` implementation.
    - Windows backend has an initial low-level hook implementation.
  - Target:
    - Use `SetWindowsHookExW(WH_KEYBOARD_LL)` for keyboard events.
    - Use `SetWindowsHookExW(WH_MOUSE_LL)` for mouse button events.
    - Emit the same normalized `input-state` payload used by macOS.
  - Remaining:
    - Build and run on a Windows machine.
    - Confirm hook lifetime and message loop behavior.
    - Confirm background capture when the app is not focused.

- [ ] Verify macOS native input backend behavior.
  - Confirm Accessibility/Input Monitoring permission requirements.
  - Confirm left/right modifier key state does not get stuck.
  - Confirm raw key codes map correctly on common keyboard layouts.
  - Confirm no crash path like the removed `rdev` dependency.

- [ ] Verify Windows overlay window behavior.
  - Transparent window background.
  - Click-through via `setIgnoreCursorEvents(true)`.
  - Always-on-top via `setAlwaysOnTop`.
  - Window sizing and position controls.

- [ ] Verify macOS overlay window behavior.
  - Transparent window background with `macOSPrivateApi`.
  - Click-through via `setIgnoreCursorEvents(true)`.
  - Always-on-top via `setAlwaysOnTop`.
  - Window sizing and position controls.

- [ ] Add platform notes to documentation.
  - Required permissions on macOS.
  - Known Windows WebView2 / transparent-window behavior.
  - Fallback behavior if global capture permission is missing.

## Commit chain plan

After cleaning up the current working tree, split the work into a logical commit
chain instead of one large mixed commit.

Suggested phases:

1. `chore: remove mobile generated project files`
   - Remove Android/iOS generated files.
   - Keep desktop-only Tauri app configuration.

2. `docs: define product direction and config format`
   - README product direction.
   - `docs/default-config.json`.
   - `docs/example-left-keyboard-config.json`.
   - TODO documentation.

3. `feat(ui): add config panel and pov overlay`
   - Replace Tauri/Vue template UI.
   - Add configuration panel.
   - Add POV overlay surface.

4. `feat(config): load and save overlay profiles`
   - Load JSON config.
   - Save & Apply.
   - Profile name handling.
   - Unit-based layout schema.

5. `feat(style): add overlay visual customization`
   - Background mode.
   - Idle key visibility.
   - Color pickers.
   - Split idle/active text colors.
   - Always-on-top and position controls.

6. `feat(input): add native input abstraction`
   - Normalized input event model.
   - Shared key mapping tests.
   - macOS backend.
   - Windows backend skeleton.

7. `test: verify build and configuration schemas`
   - Frontend tests.
   - Rust tests.
   - JSON validation.
   - Tauri build verification.
