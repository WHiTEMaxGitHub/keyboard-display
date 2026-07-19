import { describe, expect, it } from "vitest";
import {
  hexToRgb,
  normalizeHexColor,
  rgbToHex,
} from "./colorPicker";

describe("color picker utilities", () => {
  it("normalizes short and long hex colors", () => {
    expect(normalizeHexColor("#abc")).toBe("#aabbcc");
    expect(normalizeHexColor("25D366")).toBe("#25d366");
  });

  it("falls back when the input is not a hex color", () => {
    expect(normalizeHexColor("green", "#ffffff")).toBe("#ffffff");
    expect(normalizeHexColor("#12", "#000000")).toBe("#000000");
  });

  it("converts between rgb channels and hex colors", () => {
    expect(hexToRgb("#25d366")).toEqual({ r: 37, g: 211, b: 102 });
    expect(rgbToHex({ r: 37, g: 211, b: 102 })).toBe("#25d366");
  });

  it("clamps rgb channels before formatting", () => {
    expect(rgbToHex({ r: -10, g: 260, b: 20.6 })).toBe("#00ff15");
  });
});
