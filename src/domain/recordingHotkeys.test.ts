import { describe, expect, it } from "vitest";
import {
  isHotkeyMatch,
  isRecordingControlKey,
  normalizeHotkey,
  normalizeRecordingHotkeyConfig,
  recordingControlKeys,
} from "./recordingHotkeys";

describe("normalizeHotkey", () => {
  it("sorts and deduplicates keys", () => {
    expect(normalizeHotkey(["r", "ctrl-left", "r", "shift-left"])).toEqual([
      "ctrl-left",
      "shift-left",
      "r",
    ]);
  });
});

describe("recording control keys", () => {
  it("collects non-profile hotkeys so they can be filtered from recordings", () => {
    const config = {
      mode: "toggle" as const,
      start: ["f8"],
      stop: ["f8"],
      sync: ["f9"],
    };

    expect([...recordingControlKeys(config)]).toEqual(["f8", "f9"]);
    expect(isRecordingControlKey("f8", config)).toBe(true);
    expect(isRecordingControlKey("f9", config)).toBe(true);
    expect(isRecordingControlKey("w", config)).toBe(false);
  });

  it("still filters sync marker hotkey when start stop hotkeys are disabled", () => {
    const config = normalizeRecordingHotkeyConfig({
      mode: "disabled",
      sync: ["f8"],
    });

    expect([...recordingControlKeys(config)]).toEqual(["f8"]);
  });

  it("defaults sync marker hotkey to f8 for old configs", () => {
    expect(normalizeRecordingHotkeyConfig({
      mode: "toggle",
      start: [],
      stop: [],
    }).sync).toEqual(["f8"]);
  });

  it("defaults recording toggle hotkey to ctrl shift r", () => {
    expect(normalizeRecordingHotkeyConfig(undefined)).toMatchObject({
      mode: "toggle",
      start: ["ctrl-left", "shift-left", "r"],
      stop: ["ctrl-left", "shift-left", "r"],
    });
  });

  it("defaults separate stop hotkey to ctrl shift t", () => {
    expect(normalizeRecordingHotkeyConfig({ mode: "separate" })).toMatchObject({
      mode: "separate",
      start: ["ctrl-left", "shift-left", "r"],
      stop: ["ctrl-left", "shift-left", "t"],
    });
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
