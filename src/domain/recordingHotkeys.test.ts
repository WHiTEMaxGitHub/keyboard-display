import { describe, expect, it } from "vitest";
import { isHotkeyMatch, normalizeHotkey } from "./recordingHotkeys";

describe("normalizeHotkey", () => {
  it("sorts and deduplicates keys", () => {
    expect(normalizeHotkey(["r", "ctrl-left", "r", "shift-left"])).toEqual([
      "ctrl-left",
      "r",
      "shift-left",
    ]);
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

  it("does not match empty, partial, or extra key sets", () => {
    expect(isHotkeyMatch(new Set(["ctrl-left"]), [])).toBe(false);
    expect(isHotkeyMatch(new Set(["ctrl-left"]), ["ctrl-left", "r"])).toBe(false);
    expect(isHotkeyMatch(new Set(["ctrl-left", "r", "w"]), ["ctrl-left", "r"])).toBe(false);
  });
});
