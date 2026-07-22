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

- [x] Replace native `<input type="color">` with an in-app color picker.
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

- [x] Keep opacity separate from color when it affects rendering semantics.
  - Overlay opacity controls the whole POV overlay.
  - Backplate alpha can be expressed directly in the backplate color.
  - Key color alpha can be added later if needed.

## Profile editor

- [ ] Add a visual profile/config editor window.
  - Current first pass adds a Layout Editor subtab for editing existing rows.
  - Current second pass supports adding, deleting, and reordering rows/items.
  - Current third pass supports capturing a browser key into a key id.
  - TODO: Add native unknown-key fallback ids such as `macos-key-<keycode>`
    and `windows-scan-<scanCode>`, then consider a user-editable key id
    registry for naming unusual input devices.
  - Users should be able to edit `overlay.rows` without hand-writing JSON.
  - Support adding, deleting, and reordering rows.
  - Support adding, deleting, and reordering key and gap items within a row.
  - Support editing key `id`, `label`, `group`, and `widthUnit`.
  - Keep `platformLabels` behind a closed advanced editor because only a few
    keys need per-platform display names.
  - Support editing gap `widthUnit`.
  - Show a live preview using the same renderer as the POV overlay.
  - Export the result as the documented profile JSON format.
  - Keep hand-written JSON support as a first-class workflow.

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

- [x] Convert `.kbdrec` binary v1 to a frame-state stream.
  - Store the pressed-key state for sampled frames as a bitset.
  - Prefer frame-state storage over event-stream storage because replay and
    video export need final per-frame display state.
  - Extreme case is acceptable: about 108 keys at 600fps is roughly 8KB/s
    before headers or RLE.
  - Use an in-memory write buffer so recording does not perform a filesystem
    write per frame. A buffer below 10MiB is acceptable.
    Current implementation encodes into a preallocated memory buffer and writes
    the `.kbdrec` file once when recording stops.
  - Apply RLE before writing when possible:
    - `run_len varint + key_bits`
    - identical consecutive key states collapse into one run.
  - Keep input down/up events as the internal state-update source.
  - Format shape:
    - Header
    - Key table
    - RLE frame-state stream
    - Marker stream
  - Header fields:
    - magic: `"KBDR"`
    - version: `1`
    - flags
    - fps
    - key_count
    - frame_count
    - run_count
    - marker_count
  - Key table:
    - stores only real recordable keys
    - excludes virtual layout-only keys such as `void`
    - maps profile key id to compact numeric key id
  - Frame-state stream:
    - key state is a fixed-size bitset per sampled frame.
    - bit order: `byte_index = key_index / 8`, `bit_index = key_index % 8`.
    - RLE entries store `run_len varint + key_bits`.
  - Marker stream:
    - `marker_count varint`
    - each marker: `frame varint + name_len varint + utf8 name`
  - Future optional sections:
    - richer metadata for profile/layout snapshots
    - event debug stream if needed for diagnostics
  - Keep JSON/debug inspection output as a development aid.
  - Consider Huffman only as a future v2 after collecting real samples.

- [x] Add `.kbdrec` inspection / replay core.
  - Decode binary recordings into a human-readable debug view.
  - Reconstruct pressed-state frames by applying event stream changes.
  - Use this before adding video export.

- [x] Add a `.kbdrec` inspection UI or CLI entry point.
  - Choose a `.kbdrec` file.
  - Show decoded key table, events, markers, and reconstructed frames.
  - Use the same Rust inspection command as the source of truth.

- [x] Show marker metadata for editing workflows.
  - Display marker name, frame index, and timecode-style position.
  - Format marker time as `HH:MM:SS:FF @ <fps>fps`.
  - Prefer this practical metadata view before building a richer timeline.

- [x] Add a recording files browser.
  - Read the current recording save folder directly.
  - List `.kbdrec` files in a browser-style panel.
  - Show basic file information:
    - file name
    - file size
    - recording start/end time derived from the fixed filename format
    - fps
    - frame count
    - marker count
  - Click a file to expand decoded metadata.
  - Expanded metadata should include marker information in the same useful
    format as inspection:
    - marker name
    - frame index
    - `HH:MM:SS:FF @ <fps>fps`
  - Reuse the Rust `.kbdrec` inspection command as the parser.

- [x] Add root-folder creation in the recording files browser.
  - Create a new folder directly under the current recording root.
  - Refresh the recursive recording tree after creation.
  - Reject empty names and names containing path separators.
  - Let directory nodes expand and collapse using the triangle affordance.

- [x] Add sidecar metadata for recordings.
  - Store user-facing metadata outside `.kbdrec` first, likely as a JSON
    sidecar next to the recording file.
    Current implementation stores it as `<recording>.kbdrec.json`.
  - Candidate fields:
    - display name
    - description
    - tags
    - notes per marker
  - The actual `.kbdrec` filename should keep the fixed machine-friendly
    `recording-start-recording-end.kbdrec` format.

- [x] Add configurable recording display names.
  - Display name is for the browser UI only.
  - It should not rename the underlying `.kbdrec` file.
  - Use the sidecar metadata file as the storage location.

- [x] Add configurable recording filename templates later.
  - Support a VS Code-like `${...}` template style.
  - Candidate variables:
    - `${start}`
    - `${end}`
    - `${startDate}`
    - `${startTime}`
    - `${endTime}`
    - `${duration}`
    - `${profileName}`
    - `${profileSlug}`
    - `${fps}`
  - Keep this lower priority than the browser and sidecar metadata.

- [x] Support recording hotkeys outside the POV profile layout.
  - Hotkey capture should not require keys to exist in `overlay.rows`.
  - Extend normalized key mapping for common non-display hotkeys such as
    `F1` through `F12`.
  - Show hotkey labels from normalized key ids when the key is not in the
    current profile.
  - Continue filtering recording control hotkeys out of `.kbdrec` events.
  - Add tests proving a non-profile hotkey can start/stop recording without
    being recorded as input.

- [x] Use monotonic timestamps for input recordings.
  - Store event time as `t = now_monotonic - record_start_monotonic`.
  - Do not rely on wall-clock time for frame/event alignment.
  - Keep wall-clock time only for recording file names.

- [x] Add sync markers to the recording format.
  - Example event:
    - `{ "t": 18342, "type": "marker", "name": "sync" }`
  - Markers should be visible in the UI timeline/list.

- [x] Add a manual sync marker action.
  - Button: `Add sync marker`.
  - Hotkey defaults to `F8`.
  - The marker should be written into the `.kbdrec` marker stream.

- [x] Add visible sync feedback.
  - Flash the POV overlay border when a sync marker is inserted.
  - Keep the feedback small so it does not cover key states.
  - Let users disable the feedback or adjust its duration in profile recording
    config.
  - This gives the video timeline a visible cue for alignment.

- [x] Add marker timeline/editor view.
  - Show `.kbdrec` frame-state timeline with marker positions.
  - Let users inspect markers visually instead of manually calculating offsets.
  - Keep marker frame as the source of truth for future export alignment.

- [x] Add a marker rendering option for future overlay video export.
  - Store the option in profile export config.
  - Show it as a standalone export setting so marker rendering is not forced.

- [ ] Add a user-managed video exporter setup flow.
  - Use `ffmpeg` as the first video export engine, but do not assume target
    users have it installed.
  - Detect only `ffmpeg` available from `PATH` automatically.
  - Let users choose an existing `ffmpeg` application/binary path when it is
    installed outside `PATH`.
  - Offer an optional app-managed exporter install for users who do not have
    `ffmpeg` or prefer an isolated app copy.
  - Store the app-managed exporter under the app data/config directory, not in
    system paths.
  - Never modify a user's existing `ffmpeg` installation, config, shell profile,
    registry, or system `PATH`.
  - Export invocation should use the resolved binary path directly: app-managed
    path, user-selected path, or plain `ffmpeg` when `PATH` detection succeeds.
  - If the user already has `ffmpeg` in `PATH`, show it as available while still
    allowing installation of the isolated app-managed exporter.

- [ ] Render markers during future overlay video export.
  - When exporting overlay video from `.kbdrec`, optionally render visible sync
    marker feedback into the exported overlay.
  - Respect the export marker rendering option.
  - Use this for post-production alignment instead of exposing premature
    offset controls before export exists.

- [x] Document two recording workflows.
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

- [x] Add recent profile list.
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
