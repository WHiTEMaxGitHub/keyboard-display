import type { KeyBinding } from "./defaultConfig";

export type PlatformKey = "macos" | "windows" | "default";

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

  return key.platformLabels?.[platform] ?? key.label;
}
