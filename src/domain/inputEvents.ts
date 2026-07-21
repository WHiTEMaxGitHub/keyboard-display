export const INPUT_STATE_EVENT = "input-state";
export const OVERLAY_STYLE_EVENT = "overlay-style";
export const OVERLAY_VISIBLE_EVENT = "overlay-visible";
export const OVERLAY_CONFIG_EVENT = "overlay-config";
export const OVERLAY_READY_EVENT = "overlay-ready";
export const OVERLAY_SYNC_FEEDBACK_EVENT = "overlay-sync-feedback";
export const OVERLAY_ADJUST_MODE_EVENT = "overlay-adjust-mode";

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
  ["Digit7", "digit-7"],
  ["Digit8", "digit-8"],
  ["Digit9", "digit-9"],
  ["Digit0", "digit-0"],
  ["Minus", "minus"],
  ["Equal", "equal"],
  ["Tab", "tab"],
  ["KeyA", "a"],
  ["KeyB", "b"],
  ["KeyC", "c"],
  ["KeyD", "d"],
  ["KeyE", "e"],
  ["KeyF", "f"],
  ["KeyG", "g"],
  ["KeyH", "h"],
  ["KeyI", "i"],
  ["KeyJ", "j"],
  ["KeyK", "k"],
  ["KeyL", "l"],
  ["KeyM", "m"],
  ["KeyN", "n"],
  ["KeyO", "o"],
  ["KeyP", "p"],
  ["KeyQ", "q"],
  ["KeyR", "r"],
  ["KeyS", "s"],
  ["KeyT", "t"],
  ["KeyU", "u"],
  ["KeyV", "v"],
  ["KeyW", "w"],
  ["KeyX", "x"],
  ["KeyY", "y"],
  ["KeyZ", "z"],
  ["BracketLeft", "bracket-left"],
  ["BracketRight", "bracket-right"],
  ["Backslash", "backslash"],
  ["CapsLock", "caps-lock"],
  ["Semicolon", "semicolon"],
  ["Quote", "quote"],
  ["Enter", "enter"],
  ["ShiftLeft", "shift-left"],
  ["ShiftRight", "shift-right"],
  ["Comma", "comma"],
  ["Period", "period"],
  ["Slash", "slash"],
  ["ControlLeft", "ctrl-left"],
  ["ControlRight", "ctrl-right"],
  ["MetaLeft", "meta-left"],
  ["MetaRight", "meta-right"],
  ["AltLeft", "alt-left"],
  ["AltRight", "alt-right"],
  ["Space", "space"],
  ["Backspace", "backspace"],
  ["Delete", "delete"],
  ["Insert", "insert"],
  ["Home", "home"],
  ["End", "end"],
  ["PageUp", "page-up"],
  ["PageDown", "page-down"],
  ["ArrowUp", "arrow-up"],
  ["ArrowDown", "arrow-down"],
  ["ArrowLeft", "arrow-left"],
  ["ArrowRight", "arrow-right"],
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
  [1, "mouse-middle"],
  [2, "mouse-right"],
]);

export function keyIdFromKeyboardCode(code: string): string | undefined {
  return keyCodeToId.get(code);
}

export function keyIdFromMouseButton(button: number): string | undefined {
  return mouseButtonToId.get(button);
}
