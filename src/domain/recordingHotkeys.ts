export type RecordingHotkeyMode = "disabled" | "toggle" | "separate";

export type RecordingHotkeyConfig = {
  mode: RecordingHotkeyMode;
  start: string[];
  stop: string[];
  sync: string[];
};

export const DEFAULT_TOGGLE_HOTKEY = ["ctrl-left", "r", "shift-left"];
export const DEFAULT_SEPARATE_START_HOTKEY = ["ctrl-left", "r", "shift-left"];
export const DEFAULT_SEPARATE_STOP_HOTKEY = ["ctrl-left", "shift-left", "t"];
export const DEFAULT_SYNC_MARKER_HOTKEY = ["f8"];

export function normalizeRecordingHotkeyConfig(
  config: Partial<RecordingHotkeyConfig> | undefined,
): RecordingHotkeyConfig {
  const mode = config?.mode ?? "toggle";
  const start = normalizeHotkey(defaultWhenEmpty(config?.start, DEFAULT_TOGGLE_HOTKEY));
  const stop = normalizeHotkey(
    defaultWhenEmpty(config?.stop, mode === "separate" ? DEFAULT_SEPARATE_STOP_HOTKEY : start),
  );

  return {
    mode,
    start,
    stop,
    sync: normalizeHotkey(defaultWhenEmpty(config?.sync, DEFAULT_SYNC_MARKER_HOTKEY)),
  };
}

function defaultWhenEmpty(keys: string[] | undefined, fallback: string[]): string[] {
  return keys && keys.length > 0 ? keys : fallback;
}

export function normalizeHotkey(keys: Iterable<string>): string[] {
  return [...new Set(keys)].sort(compareHotkeyKey);
}

function compareHotkeyKey(left: string, right: string): number {
  const priority = (key: string) => {
    if (key.startsWith("ctrl-")) return 0;
    if (key.startsWith("shift-")) return 1;
    if (key.startsWith("alt-")) return 2;
    if (key.startsWith("meta-")) return 3;
    return 4;
  };

  return priority(left) - priority(right) || left.localeCompare(right);
}

export function isHotkeyMatch(activeKeys: Set<string>, hotkey: string[]): boolean {
  const normalizedHotkey = normalizeHotkey(hotkey);

  if (normalizedHotkey.length === 0) {
    return false;
  }

  return normalizedHotkey.every((key) => activeKeys.has(key));
}

export function recordingControlKeys(config: RecordingHotkeyConfig): Set<string> {
  if (config.mode === "disabled") {
    return new Set(config.sync);
  }

  return new Set([...config.start, ...config.stop, ...config.sync]);
}

export function isRecordingControlKey(keyId: string, config: RecordingHotkeyConfig): boolean {
  return recordingControlKeys(config).has(keyId);
}
