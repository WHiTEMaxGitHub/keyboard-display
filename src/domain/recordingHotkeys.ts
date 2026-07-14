export type RecordingHotkeyMode = "disabled" | "toggle" | "separate";

export type RecordingHotkeyConfig = {
  mode: RecordingHotkeyMode;
  start: string[];
  stop: string[];
};

export function normalizeHotkey(keys: Iterable<string>): string[] {
  return [...new Set(keys)].sort();
}

export function isHotkeyMatch(activeKeys: Set<string>, hotkey: string[]): boolean {
  const normalizedHotkey = normalizeHotkey(hotkey);

  if (normalizedHotkey.length === 0) {
    return false;
  }

  if (activeKeys.size !== normalizedHotkey.length) {
    return false;
  }

  return normalizedHotkey.every((key) => activeKeys.has(key));
}
