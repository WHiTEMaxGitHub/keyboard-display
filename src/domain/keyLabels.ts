import type { KeyBinding } from "./defaultConfig";

export type PlatformKey = "macos" | "windows" | "default";

const builtInPlatformLabels: Record<string, Partial<Record<PlatformKey, string>>> = {
  "alt-left": {
    macos: "OptionLeft",
    windows: "AltLeft",
  },
  "alt-right": {
    macos: "OptionRight",
    windows: "AltRight",
  },
  "meta-left": {
    macos: "Cmd",
    windows: "Win",
  },
  "meta-right": {
    macos: "Cmd",
    windows: "Win",
  },
};

export function detectPlatformKey(platform: string = navigator.platform): PlatformKey {
  const normalizedPlatform = platform.toLowerCase();

  if (normalizedPlatform.includes("mac")) {
    return "macos";
  }

  if (normalizedPlatform.includes("win")) {
    return "windows";
  }

  return "default";
}

export function displayLabelForKey(key: KeyBinding, platform: PlatformKey): string {
  if (platform === "default") {
    return key.label;
  }

  return key.platformLabels?.[platform] ?? builtInPlatformLabels[key.id]?.[platform] ?? key.label;
}
