# JSON configuration format

This document describes the JSON files that users can edit by hand.

There are two configuration levels:

- Profile config: shareable POV overlay layout and style. Examples:
  - `docs/default-config.json`
  - `docs/example-left-keyboard-config.json`
- App config: local application state. Example:
  - `docs/app-config.json`

Profile configs are the files users should usually share or hand edit. App
config stores local startup state and should normally be managed by the app.

## Profile config

Profile config files use this top-level shape:

```json
{
  "version": 1,
  "name": "CS POV",
  "overlay": {},
  "recording": {},
  "export": {}
}
```

### Top-level fields

| Field | Type | Meaning |
| --- | --- | --- |
| `version` | number | Config schema version. Current value is `1`. |
| `name` | string | Profile name shown in the configuration UI. |
| `overlay` | object | POV overlay visibility, position, visual style, rows, and layout units. |
| `recording` | object | Default recording settings for this profile. |
| `export` | object | Preferred future video export formats. |

## `overlay`

```json
{
  "visible": true,
  "position": "bottom-right",
  "layout": {
    "unitPx": 54,
    "gapUnit": 0.15
  },
  "style": {},
  "rows": []
}
```

| Field | Type | Meaning |
| --- | --- | --- |
| `visible` | boolean | Whether the POV overlay should be visible after loading this profile. |
| `position` | string | Initial overlay position. Supported values are `top-left`, `top-right`, `bottom-left`, `bottom-right`, and `center`. |
| `layout.unitPx` | number | Pixel size of one layout unit before `style.scale` is applied. |
| `layout.gapUnit` | number | Default gap between two adjacent normal keys, measured in layout units. |
| `style` | object | Overlay colors, opacity, background mode, and window behavior. |
| `rows` | array | Visual keyboard layout. Each entry is one rendered row. |

Unit counts are normalized to two decimal places when loaded. For example,
`1.236` becomes `1.24`.

## Rows

`overlay.rows` is the source of truth for the visual layout. It is an array of
rows, and each row is an array of items.

```json
"rows": [
  [
    { "type": "key", "id": "a", "label": "A", "group": "movement", "widthUnit": 1 },
    { "type": "key", "id": "s", "label": "S", "group": "movement", "widthUnit": 1 }
  ],
  [
    { "type": "key", "id": "space", "label": "Space", "group": "modifier", "widthUnit": 4 }
  ]
]
```

### Key item

```json
{
  "type": "key",
  "id": "w",
  "label": "W",
  "group": "movement",
  "widthUnit": 1,
  "platformLabels": {
    "macos": "Cmd",
    "windows": "Win"
  }
}
```

| Field | Type | Meaning |
| --- | --- | --- |
| `type` | string | Use `key` for a real recordable/displayed key. Existing configs may omit this; omitted means `key`. |
| `id` | string | Stable normalized input id. This is used for matching native input events and recording. |
| `label` | string | Default text shown on the key. |
| `group` | string | Logical category. Current values are `mouse`, `movement`, `modifier`, and `action`. |
| `widthUnit` | number | Key width measured in layout units. `1` is the normal key width. |
| `platformLabels` | object | Optional per-platform display labels. Supported keys are `macos` and `windows`. |

`id` must match the normalized input id emitted by the app. Common examples:

- Keyboard: `w`, `a`, `s`, `d`, `q`, `e`, `r`, `space`, `tab`, `escape`,
  `shift-left`, `ctrl-left`, `alt-left`, `meta-left`, `caps-lock`,
  `backquote`, `digit-1`
- Mouse: `mouse-left`, `mouse-right`, `mouse-middle`

### Gap item

```json
{
  "type": "gap",
  "widthUnit": 0.5
}
```

`gap` is a virtual layout item. It is not recorded and it does not respond to
input. Use it to place a custom blank space inside a row.

Gap behavior:

- If two normal key items are adjacent, the app inserts the default
  `overlay.layout.gapUnit` between them.
- If a `gap` item is placed between keys, the app uses only that gap item's
  `widthUnit`.
- A custom `gap` does not also receive the default `gapUnit` on its left or
  right.
- A `gap` at the start or end of a row is allowed and only occupies its own
  `widthUnit`.

`void` is accepted as an alias-like virtual item type in runtime code, but
profile files should prefer `gap` for clarity.

## `style`

```json
{
  "scale": 1,
  "opacity": 0.92,
  "backgroundMode": "transparent",
  "backgroundColor": "#0a0c0e",
  "backgroundOpacity": 0.72,
  "backgroundRadius": 8,
  "idleKeyVisibility": "visible",
  "alwaysOnTop": false,
  "idleColor": "#121417",
  "activeColor": "#25d366",
  "idleTextColor": "#f5f7fa",
  "activeTextColor": "#ffffff"
}
```

| Field | Type | Meaning |
| --- | --- | --- |
| `scale` | number | Multiplies the whole overlay size. |
| `opacity` | number | Opacity for the whole overlay. Range is usually `0` to `1`. |
| `backgroundMode` | string | `transparent` or `panel`. |
| `backgroundColor` | string | Backplate color as a hex color. |
| `backgroundOpacity` | number | Backplate opacity when `backgroundMode` is `panel`. |
| `backgroundRadius` | number | Backplate corner radius in pixels. |
| `idleKeyVisibility` | string | `visible`, `faint`, or `hidden`. |
| `alwaysOnTop` | boolean | Whether the POV window should stay above normal windows. |
| `idleColor` | string | Key background color when not pressed. |
| `activeColor` | string | Key background color when pressed. |
| `idleTextColor` | string | Text color when not pressed. |
| `activeTextColor` | string | Text color when pressed. |

## `recording`

```json
{
  "defaultFps": 60,
  "fpsOptions": [30, 60, 120],
  "customFpsEnabled": false,
  "customFps": 600,
  "maxFps": 1000,
  "syncFeedbackEnabled": true,
  "syncFeedbackDurationMs": 420,
  "formatExtension": ".kbdrec",
  "primaryArtifact": "input-binary"
}
```

| Field | Type | Meaning |
| --- | --- | --- |
| `defaultFps` | number | Default capture frame rate. |
| `fpsOptions` | number[] | Preset frame rates shown in the UI. |
| `customFpsEnabled` | boolean | Whether to use `customFps` instead of `defaultFps`. |
| `customFps` | number | User-defined capture frame rate. |
| `maxFps` | number | Maximum allowed capture frame rate. Current default is `1000`. |
| `syncFeedbackEnabled` | boolean | Whether sync markers should briefly flash the POV overlay border. |
| `syncFeedbackDurationMs` | number | Border flash duration in milliseconds. |
| `formatExtension` | string | Recording extension. Current value is `.kbdrec`. |
| `primaryArtifact` | string | Recording strategy. Current value is `input-binary`. |

## `export`

```json
{
  "defaultFormat": "webm",
  "transparentFormat": "webm",
  "compatibleFormat": "mp4"
}
```

These fields describe future export preferences. Video export is not the source
recording format; `.kbdrec` is the source of truth.

## App config

`docs/app-config.json` documents local application state:

```json
{
  "version": 1,
  "profiles": {},
  "currentProfile": {},
  "recording": {},
  "ui": {}
}
```

| Field | Type | Meaning |
| --- | --- | --- |
| `profiles.defaultProfilePath` | string | Default profile path used by the app. |
| `profiles.lastProfilePath` | string or null | Last loaded profile file path. |
| `profiles.recentProfiles` | array | Recent profile list for future quick switching. |
| `currentProfile` | object | Current active profile state. Its `overlay` and `recording` use the same schema as profile configs. |
| `currentProfile.changed` | boolean | Whether current profile state differs from `currentProfile.sourcePath`. |
| `recording.outputDirectory` | string or null | Folder for new `.kbdrec` recordings. |
| `recording.silent` | boolean | Whether recording should destroy the POV overlay window while recording. |
| `recording.hotkeys` | object | Recording hotkey settings. |
| `ui.language` | string | Future UI language setting. `system` means follow system/default behavior. |

Users should usually edit profile config files rather than app config. App
config may contain local absolute paths.

## Minimal profile example

```json
{
  "version": 1,
  "name": "Minimal WASD",
  "overlay": {
    "visible": true,
    "position": "bottom-right",
    "layout": { "unitPx": 54, "gapUnit": 0.15 },
    "style": {
      "scale": 1,
      "opacity": 0.92,
      "backgroundMode": "transparent",
      "backgroundColor": "#0a0c0e",
      "backgroundOpacity": 0.72,
      "backgroundRadius": 8,
      "idleKeyVisibility": "visible",
      "alwaysOnTop": false,
      "idleColor": "#121417",
      "activeColor": "#25d366",
      "idleTextColor": "#f5f7fa",
      "activeTextColor": "#ffffff"
    },
    "rows": [
      [
        { "type": "gap", "widthUnit": 1 },
        { "type": "key", "id": "w", "label": "W", "group": "movement", "widthUnit": 1 }
      ],
      [
        { "type": "key", "id": "a", "label": "A", "group": "movement", "widthUnit": 1 },
        { "type": "key", "id": "s", "label": "S", "group": "movement", "widthUnit": 1 },
        { "type": "key", "id": "d", "label": "D", "group": "movement", "widthUnit": 1 }
      ]
    ]
  },
  "recording": {
    "defaultFps": 60,
    "fpsOptions": [30, 60, 120],
    "customFpsEnabled": false,
    "customFps": 600,
    "maxFps": 1000,
    "syncFeedbackEnabled": true,
    "syncFeedbackDurationMs": 420,
    "formatExtension": ".kbdrec",
    "primaryArtifact": "input-binary"
  },
  "export": {
    "defaultFormat": "webm",
    "transparentFormat": "webm",
    "compatibleFormat": "mp4"
  }
}
```
