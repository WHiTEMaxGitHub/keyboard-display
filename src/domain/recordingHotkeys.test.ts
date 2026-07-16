import { describe, expect, it } from "vitest";
import {
  isHotkeyMatch,
  isRecordingControlKey,
  normalizeHotkey,
  recordingControlKeys,
} from "./recordingHotkeys";

describe("normalizeHotkey", () => {
  it("sorts and deduplicates keys", () => {
    expect(normalizeHotkey(["r", "ctrl-left", "r", "shift-left"])).toEqual([
      "ctrl-left",
      "r",
      "shift-left",
    ]);
  });
});

describe("recording control keys", () => {
  it("collects non-profile hotkeys so they can be filtered from recordings", () => {
    const config = {
      mode: "toggle" as const,
      start: ["f8"],
      stop: ["f8"],
    };

    expect([...recordingControlKeys(config)]).toEqual(["f8"]);
    expect(isRecordingControlKey("f8", config)).toBe(true);
    expect(isRecordingControlKey("w", config)).toBe(false);
  });
});

describe("isHotkeyMatch", () => {
  it("matches the exact active key set", () => {
    expect(
      isHotkeyMatch(new Set(["ctrl-left", "shift-left", "r"]), [
        "shift-left",
        "ctrl-left",
        "r",
      ]),
    ).toBe(true);
  });

  it("matches when the active key set contains the whole hotkey", () => {
    expect(isHotkeyMatch(new Set(["ctrl-left", "shift-left", "r", "w"]), [
      "shift-left",
      "ctrl-left",
      "r",
    ])).toBe(true);
  });

  it("does not match empty or partial key sets", () => {
    expect(isHotkeyMatch(new Set(["ctrl-left"]), [])).toBe(false);
    expect(isHotkeyMatch(new Set(["ctrl-left"]), ["ctrl-left", "r"])).toBe(false);
  });
});
