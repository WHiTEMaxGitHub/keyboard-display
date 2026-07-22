# Keyboard Display

Keyboard Display is a desktop key input overlay for game POV videos. The goal is
to show pressed keys in real time, similar to the keyboard indicators often seen
in CS POV content.

The project is desktop-first. It does not target iOS or Android because the core
features depend on desktop global input capture, overlay windows, and video
export workflows.

## Product Direction

The app should provide:

- A transparent or compact desktop overlay for keys such as `W`, `A`, `S`, `D`,
  `Space`, `Shift`, `Ctrl`, mouse buttons, and other game-specific controls.
- Global keyboard and mouse capture from the native desktop layer.
- A configurable visual layout for streamers, players, and video editors.
- A recording format that stores input data, not rendered video frames.
- Replay and video export based on the recorded input timeline.

## Global Input Capture

Global input capture is a core feature and must be designed as a cross-platform
native layer. The project should not depend on a macOS-only implementation or a
library path that hides platform-specific behavior behind one unstable API.

The intended shape is:

```text
native input backend -> normalized input event -> overlay renderer / recorder
```

Requirements:

- Support at least macOS and Windows.
- Keep a shared normalized event model for keys and mouse buttons.
- Implement platform backends behind a common Rust interface.
- On macOS, avoid libraries that convert keyboard codes through AppKit or input
  source APIs from background hook threads.
- On Windows, use the appropriate native low-level keyboard and mouse hooks.
- Treat OS permissions as part of setup: macOS may require Accessibility/Input
  Monitoring permission for real background capture.

Until the platform backends are implemented, frontend key events are only a
preview mechanism and do not represent true background capture.

## Recording Strategy

The main product difference is the save format. The app should not save raw
overlay video as the primary artifact. Instead, it should save a compact binary
recording file made of input frames.

The user chooses a capture frame rate, such as `30fps`, `60fps`, or `120fps`.
Profiles may also enable a custom capture frame rate up to `1000fps`. Each
captured frame stores the current input state:

- key states as bitsets;
- mouse button states as bitsets;
- frame index or timestamp;
- optional future extensions such as mouse movement or layout metadata.

This keeps the original recording small. For example, if 64 keys fit in an
8-byte bitset, a 60fps recording needs only about 28.8KB per minute for the core
key state stream, before headers and optional metadata.

## Replay And Export

The binary input recording is the source of truth:

```text
.kbdrec -> replay engine -> overlay renderer -> video encoder
```

This design allows the same recording to be rendered later with different
layouts, colors, sizes, transparency settings, or output formats. Video should
be treated as an export result, not the original save format.

For video export, the app should prioritize low file size:

- configurable output frame rate;
- configurable resolution and bitrate;
- transparent WebM when alpha is needed;
- MP4/H.264 when compatibility matters more;
- no full video frame storage in the primary recording file.

## Development

This project uses Tauri, Vue, TypeScript, and Vite.

The default POV profile shape is documented in
[`docs/default-config.json`](docs/default-config.json). The app-level
configuration shape is documented in [`docs/app-config.json`](docs/app-config.json).
For hand-written profile files, see
[`docs/config-format.md`](docs/config-format.md).
Recording workflows are documented in
[`docs/recording-workflows.md`](docs/recording-workflows.md).

Layout sizes use project-defined units. `overlay.layout.unitPx` defines the
pixel size of one unit, while each key's `widthUnit` and the layout `gapUnit`
store unit counts. Unit counts are normalized to at most two decimal places.

Use the Node version declared in `.nvmrc`:

```sh
nvm use
pnpm install
pnpm tauri dev
```
