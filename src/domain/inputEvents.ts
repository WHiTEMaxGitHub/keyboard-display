export const INPUT_STATE_EVENT = "input-state";
export const OVERLAY_STYLE_EVENT = "overlay-style";
export const OVERLAY_VISIBLE_EVENT = "overlay-visible";
export const OVERLAY_CONFIG_EVENT = "overlay-config";

export type InputStatePayload = {
  keyId: string;
  pressed: boolean;
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
