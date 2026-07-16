import { describe, expect, it } from "vitest";
import { keyIdFromKeyboardCode, keyIdFromMouseButton } from "./inputEvents";

describe("input event mapping", () => {
  it("maps keyboard codes to overlay key ids", () => {
    expect(keyIdFromKeyboardCode("Escape")).toBe("escape");
    expect(keyIdFromKeyboardCode("Backquote")).toBe("backquote");
    expect(keyIdFromKeyboardCode("Digit1")).toBe("digit-1");
    expect(keyIdFromKeyboardCode("KeyW")).toBe("w");
    expect(keyIdFromKeyboardCode("KeyT")).toBe("t");
    expect(keyIdFromKeyboardCode("CapsLock")).toBe("caps-lock");
    expect(keyIdFromKeyboardCode("KeyF")).toBe("f");
    expect(keyIdFromKeyboardCode("ShiftLeft")).toBe("shift-left");
    expect(keyIdFromKeyboardCode("ControlLeft")).toBe("ctrl-left");
    expect(keyIdFromKeyboardCode("MetaLeft")).toBe("meta-left");
    expect(keyIdFromKeyboardCode("AltLeft")).toBe("alt-left");
    expect(keyIdFromKeyboardCode("Space")).toBe("space");
    expect(keyIdFromKeyboardCode("F1")).toBe("f1");
    expect(keyIdFromKeyboardCode("F12")).toBe("f12");
  });

  it("maps mouse buttons to overlay key ids", () => {
    expect(keyIdFromMouseButton(0)).toBe("mouse-left");
    expect(keyIdFromMouseButton(2)).toBe("mouse-right");
    expect(keyIdFromMouseButton(1)).toBeUndefined();
  });
});
