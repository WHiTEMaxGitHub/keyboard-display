export const INPUT_STATE_EVENT = "input-state";
export const OVERLAY_STYLE_EVENT = "overlay-style";
export const OVERLAY_VISIBLE_EVENT = "overlay-visible";
export const OVERLAY_CONFIG_EVENT = "overlay-config";
export const OVERLAY_SYNC_FEEDBACK_EVENT = "overlay-sync-feedback";
export const OVERLAY_MEASURED_EVENT = "overlay-measured";
export const OVERLAY_ADJUST_MODE_EVENT = "overlay-adjust-mode";

export type InputStatePayload = {
  keyId: string;
  pressed: boolean;
};

export type OverlayMeasuredPayload = {
  width: number;
  height: number;
};

const keyCodeToId = new Map<string, string>([
  ["Escape", "escape"],
  ["Backquote", "backquote"],
  ["Digit1", "digit-1"],
  ["Digit2", "digit-2"],
  ["Digit3", "digit-3"],
  ["Digit4", "digit-4"],
  ["Digit5", "digit-5"],
  ["Digit6", "digit-6"],
  ["Tab", "tab"],
  ["KeyW", "w"],
  ["KeyA", "a"],
  ["KeyS", "s"],
  ["KeyD", "d"],
  ["KeyT", "t"],
  ["CapsLock", "caps-lock"],
  ["KeyF", "f"],
  ["KeyG", "g"],
  ["ShiftLeft", "shift-left"],
  ["ShiftRight", "shift-right"],
  ["KeyZ", "z"],
  ["KeyX", "x"],
  ["KeyC", "c"],
  ["KeyV", "v"],
  ["KeyB", "b"],
  ["ControlLeft", "ctrl-left"],
  ["ControlRight", "ctrl-right"],
  ["MetaLeft", "meta-left"],
  ["AltLeft", "alt-left"],
  ["Space", "space"],
  ["KeyR", "r"],
  ["KeyQ", "q"],
  ["KeyE", "e"],
  ["F1", "f1"],
  ["F2", "f2"],
  ["F3", "f3"],
  ["F4", "f4"],
  ["F5", "f5"],
  ["F6", "f6"],
  ["F7", "f7"],
  ["F8", "f8"],
  ["F9", "f9"],
  ["F10", "f10"],
  ["F11", "f11"],
  ["F12", "f12"],
]);

const mouseButtonToId = new Map<number, string>([
  [0, "mouse-left"],
  [2, "mouse-right"],
]);

export function keyIdFromKeyboardCode(code: string): string | undefined {
  return keyCodeToId.get(code);
}

export function keyIdFromMouseButton(button: number): string | undefined {
  return mouseButtonToId.get(button);
}
