import { describe, expect, it } from "vitest";
import { resolveCustomOverlayPosition } from "./overlayPosition";

describe("overlay position", () => {
  it("maps custom position between monitor work areas by ratio", () => {
    const position = resolveCustomOverlayPosition(
      {
        x: 3200,
        y: 1800,
        workArea: {
          x: 0,
          y: 0,
          width: 3840,
          height: 2160,
        },
        scaleFactor: 1,
      },
      {
        x: 0,
        y: 0,
        width: 1512,
        height: 982,
      },
      {
        width: 300,
        height: 120,
      },
    );

    expect(position).toEqual({
      x: 1212,
      y: 818,
    });
  });

  it("clamps stale custom position into current work area", () => {
    const position = resolveCustomOverlayPosition(
      {
        x: 4000,
        y: 2200,
      },
      {
        x: 0,
        y: 0,
        width: 1512,
        height: 982,
      },
      {
        width: 300,
        height: 120,
      },
    );

    expect(position).toEqual({
      x: 1212,
      y: 862,
    });
  });
});
