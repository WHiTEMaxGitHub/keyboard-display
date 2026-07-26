import type { KeyBinding, KeyIdLabelRegistry } from "./defaultConfig";

export type PlatformKey = "macos" | "windows" | "default";

const builtInPlatformLabels: Record<string, Partial<Record<PlatformKey, string>>> = {
  "alt-left": {
    macos: "Cmd",
    windows: "AltLeft",
  },
  "alt-right": {
    macos: "Cmd",
    windows: "AltRight",
  },
  "meta-left": {
    macos: "Opt",
    windows: "Win",
  },
  "meta-right": {
    macos: "Opt",
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

export function displayLabelForKey(
  key: KeyBinding,
  platform: PlatformKey,
  keyIdLabels: KeyIdLabelRegistry = {},
): string {
  const registryLabel = keyIdLabels[key.id];
  if (registryLabel && isPlaceholderLabel(key.label)) {
    return registryLabel;
  }

  if (platform === "default") {
    return key.label;
  }

  return key.platformLabels?.[platform] ?? builtInPlatformLabels[key.id]?.[platform] ?? key.label;
}

function isPlaceholderLabel(label: string) {
  return label.trim() === "" || label.trim().toLowerCase() === "key";
}
