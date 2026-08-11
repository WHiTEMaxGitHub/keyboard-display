# Keyboard Display

Keyboard Display is a desktop POV key overlay and recording tool. It captures keyboard and mouse state through native input backends, renders a live key overlay in a separate transparent window, stores the input timeline as compact `.kbdrec` files, and can later render those recordings into transparent overlay video.

Chinese README: [`README.md`](README.md).

## Current Features

- Desktop Tauri app with Vue 3, TypeScript, Vite, and Tailwind CSS.
- Custom-drawn configuration window title bar, with macOS and Windows controls placed according to each platform's convention.
- Separate POV Overlay window with transparent background, always-on-top support, position adjustment, and custom layouts.
- Native global input capture:
  - macOS uses CGEventTap.
  - Windows uses Raw Input for keyboard input and a low-level mouse hook for mouse buttons.
  - Other platforms start an unsupported backend. The configuration UI can still open, but real background input capture is unavailable.
- Visual configuration pages:
  - Overview: current config state, live preview, visibility and always-on-top shortcuts.
  - Layout: summary and editor sub-pages for rows, keys, gaps, Key ID, display name, width, `unitPx`, and `gapUnit`.
  - Appearance: overlay scale, opacity, backplate, radius, key colors, and transparent backplate.
  - Window: visibility, always-on-top, preset positions, and visual adjustment.
  - Recording: save folder, FPS, filename template, sync flash, recording hotkeys, and background recording.
  - Recording Files: recursive `.kbdrec` browser, folder creation, recording inspection, and sidecar metadata editing.
  - Export: choose `.kbdrec`, export overlay WebM, select or install ffmpeg, configure font and render threads.
  - Settings: language, theme, custom colors, app config path, and runtime state.
- Theme system:
  - Built-in `vibrant`, `dark`, `midnight`, `light`, and `custom` themes.
  - Custom themes support background templates, color channels, HEX/RGBA color picking, and panel opacity.
- English and Simplified Chinese UI, with `system`, `en`, and `zh-CN` language modes.

## Input And Overlay

Input events are normalized by the Rust native layer into stable Key IDs, then sent to the configuration window and POV Overlay:

```text
native input backend -> normalized input event -> config window / POV overlay / recorder
```

Common Key ID examples:

- Keyboard: `w`, `a`, `s`, `d`, `space`, `shift-left`, `ctrl-left`, `alt-left`, `meta-left`, `escape`
- Mouse: `mouse-left`, `mouse-right`, `mouse-middle`

On macOS, real background capture usually requires Accessibility/Input Monitoring permissions. Platform details are documented in [`docs/platform-notes.md`](docs/platform-notes.md).

## Configuration Model

The project uses two JSON levels:

- Profile config: shareable and hand-editable POV config, containing layout, appearance, recording defaults, and export preferences.
- App config: local application state, containing the active config snapshot, current profile file path, recording folder, export folder, hotkeys, language, and related state.

Built-in profiles live in code. Users do not need to receive a separate bundle of JSON files with the app. Current built-in templates are:

- `default`
- `left-keyboard`
- `68-keyboard`

In release builds, the app looks for `app-config.json` next to the application executable. If the file is missing or contains invalid JSON, the app regenerates it from the built-in initial config. Debug builds keep using the platform app config directory and do not create a config next to the development binary.

The initial `app-config.json` intentionally does not prefill user storage paths:

- `currentProfile.sourcePath` starts as `null`. After the user exports or loads a profile JSON, it stores the current profile file path so "Overwrite & Apply" can save edits back to disk.
- The recording output directory starts as `null`. If the user starts recording without choosing a folder, the app uses `recording-files` next to app config.
- The overlay video export directory starts as `null`. If the user exports video without choosing a folder, the app uses `export-videos` next to app config.

The app does not create a `pov-profiles` folder on startup. Profile JSON files are created only when the user exports or saves a config and chooses a file path.

The config format is documented in [`docs/config-format.md`](docs/config-format.md). Examples are available in [`docs/default-config.json`](docs/default-config.json) and [`docs/app-config.json`](docs/app-config.json).

## Recording

Recording files use the `.kbdrec` extension. They store the input-state timeline, not rendered video frames:

```text
input events -> sampled frames -> .kbdrec
```

Recording options include:

- Preset FPS values: `30`, `60`, `120`
- Custom FPS up to `1000`
- Filename template, defaulting to `${start}-${end}`
- Sync markers and sync flash
- Recording control hotkeys:
  - Default start/stop: `ctrl-left + shift-left + r`
  - Default sync marker: `f8`
- Background recording: destroys the POV window during recording and restores it afterward.

The Recording Files page can read `.kbdrec` summaries, events, frame states, and markers. It also stores sidecar metadata such as display name, description, tags, and marker notes.

## Export

`.kbdrec` is the source of truth, and overlay video is an export result:

```text
.kbdrec -> replay engine -> overlay renderer -> ffmpeg -> .webm
```

The current export target is transparent overlay WebM. The Export page detects three ffmpeg sources:

- App-managed ffmpeg.
- User-selected ffmpeg.
- ffmpeg from system PATH.

The app never modifies the user's existing ffmpeg installation. App-managed ffmpeg is installed under the app data directory's exporter subdirectory.

Render thread count is controlled by profile `export.renderThreads`:

- Empty or `0`: use the current CPU core count.
- `-1`: high concurrency mode, equal to CPU cores multiplied by `4`.
- Positive value: use the requested worker count, capped at CPU cores multiplied by `4`.

The export output directory starts empty. If the user exports without choosing a folder, the app uses `export-videos` next to app config.

## Runtime Files

In release builds, if the user does not manually choose any folders, the application directory typically becomes:

```text
Keyboard Display directory/
├─ app-config.json
├─ recording-files/
│  └─ *.kbdrec
└─ export-videos/
   └─ *.webm
```

Exported profile JSON files can live anywhere chosen by the system save dialog. `app-config.json` stores only the current profile `sourcePath`; it does not keep recent-profile history.

## Development

Use the Node version declared in `.nvmrc`:

```sh
nvm use
pnpm install
pnpm tauri dev
```

Common commands:

```sh
pnpm test
pnpm build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

Start only the Vite frontend:

```sh
pnpm dev
```

Build the Tauri app:

```sh
pnpm tauri build
```
