import { describe, expect, it } from "vitest";
import { detectPlatformKey, displayLabelForKey } from "./keyLabels";

describe("detectPlatformKey", () => {
  it("detects macOS and Windows platform labels", () => {
    expect(detectPlatformKey("MacIntel")).toBe("macos");
    expect(detectPlatformKey("Win32")).toBe("windows");
    expect(detectPlatformKey("Linux x86_64")).toBe("default");
  });
});

describe("displayLabelForKey", () => {
  it("uses platform-specific labels when present", () => {
    const key = {
      id: "meta-left",
      label: "Meta",
      group: "modifier" as const,
      row: 0,
      widthUnit: 1,
      platformLabels: {
        macos: "Cmd",
        windows: "Win",
      },
    };

    expect(displayLabelForKey(key, "macos")).toBe("Cmd");
    expect(displayLabelForKey(key, "windows")).toBe("Win");
    expect(displayLabelForKey(key, "default")).toBe("Meta");
  });

  it("uses built-in platform labels for common modifiers", () => {
    expect(displayLabelForKey({
      id: "alt-right",
      label: "Alt",
      group: "modifier",
      widthUnit: 1,
    }, "macos")).toBe("OptionRight");
    expect(displayLabelForKey({
      id: "alt-right",
      label: "Alt",
      group: "modifier",
      widthUnit: 1,
    }, "windows")).toBe("AltRight");
    expect(displayLabelForKey({
      id: "meta-left",
      label: "Meta",
      group: "modifier",
      widthUnit: 1,
    }, "windows")).toBe("Win");
  });
});
