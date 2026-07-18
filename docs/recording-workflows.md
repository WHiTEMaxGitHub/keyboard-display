# Recording workflows

Keyboard Display stores `.kbdrec` files as the source recording. Video is either
captured live together with the POV overlay, or generated later from the
recording timeline.

## Realtime OBS capture

Use this workflow when the POV overlay should appear directly in the gameplay
recording.

1. Load or edit the POV profile.
2. Keep `Show POV overlay` enabled.
3. Keep `Silent recording` disabled.
4. Start OBS or another screen recorder.
5. Start Keyboard Display recording.
6. Press `Add sync marker` or the sync hotkey when an alignment point is needed.
7. Stop Keyboard Display recording and stop the screen recorder.

In this workflow, sync markers are useful because the overlay border flashes in
the captured video. The `.kbdrec` file also stores the marker frame, so the same
point is visible in both the video and the input timeline.

## Post-production export

Use this workflow when the game video is recorded separately and the key overlay
should be rendered later.

1. Load or edit the POV profile.
2. Choose a recording folder, or use the default app recording folder.
3. Enable `Silent recording` if the live POV window should not consume rendering
   resources during gameplay.
4. Start Keyboard Display recording.
5. Record gameplay with the preferred capture tool.
6. Press `Add sync marker` or the sync hotkey at visible moments that can be
   found in the game footage.
7. Stop Keyboard Display recording.
8. Inspect the `.kbdrec` file and use marker frames to align the future overlay
   export with the gameplay footage.

In this workflow, `.kbdrec` remains the source of truth. A later export tool
should render the overlay from frame-state data. Marker frames should be shown
visually in an editor/timeline so users can align the generated overlay with
external footage without doing manual offset math in the inspection view.

## Marker behavior

Markers are frame-based. The `.kbdrec` file stores marker frame indexes rather
than wall-clock timestamps.

The sync marker hotkey defaults to `F8`. It is treated as a control input and is
not written into the key-state stream. Start and stop hotkeys are also filtered
only when the complete hotkey is triggered; individual keys such as `Ctrl`,
`Shift`, `R`, or `T` are still recorded when pressed by themselves.

## Recording folders

If no recording folder is selected, the app uses an app-local default
`recording-files` folder. The Recording page shows the actual path before
recording starts.
