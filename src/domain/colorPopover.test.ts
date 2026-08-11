import { describe, expect, it } from "vitest";
import { placeColorPopover } from "./colorPopover";

describe("color popover placement", () => {
  it("opens below the trigger when there is enough viewport space", () => {
    expect(
      placeColorPopover(
        { left: 120, right: 240, top: 100, bottom: 138, width: 120 },
        { width: 900, height: 700 },
      ),
    ).toEqual({
      direction: "down",
      left: 120,
      top: 146,
      width: 260,
      maxHeight: 360,
    });
  });

  it("opens above the trigger when the bottom edge would clip the panel", () => {
    expect(
      placeColorPopover(
        { left: 120, right: 240, top: 580, bottom: 618, width: 120 },
        { width: 900, height: 700 },
        { panelHeight: 312 },
      ),
    ).toEqual({
      direction: "up",
      left: 120,
      top: 260,
      width: 260,
      maxHeight: 360,
    });
  });

  it("keeps the panel inside the horizontal viewport margins", () => {
    expect(
      placeColorPopover(
        { left: 820, right: 880, top: 100, bottom: 138, width: 60 },
        { width: 900, height: 700 },
      ).left,
    ).toBe(624);
  });
});
