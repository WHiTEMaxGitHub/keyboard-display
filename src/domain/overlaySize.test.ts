import { describe, expect, it } from "vitest";
import { createDefaultConfig } from "./defaultConfig";
import { estimateOverlaySize } from "./overlaySize";

describe("estimateOverlaySize", () => {
  it("scales default row overlay dimensions", () => {
    const config = createDefaultConfig();
    const normal = estimateOverlaySize(config.layout, config.rows, config.style);
    const large = estimateOverlaySize(config.layout, config.rows, {
      ...config.style,
      scale: 2,
    });

    expect(large.width).toBeGreaterThan(normal.width);
    expect(large.height).toBeGreaterThan(normal.height);
  });

  it("includes the transparent window padding", () => {
    const config = createDefaultConfig();
    const size = estimateOverlaySize(config.layout, config.rows, config.style);

    expect(size.height).toBeGreaterThan(5 * config.layout.unitPx);
  });

  it("uses default gap only between adjacent keys", () => {
    const config = createDefaultConfig();
    const size = estimateOverlaySize(
      {
        unitPx: 100,
        gapUnit: 0.2,
      },
      [
        [
          { type: "key", id: "a", label: "A", group: "movement", widthUnit: 1 },
          { type: "key", id: "s", label: "S", group: "movement", widthUnit: 1 },
        ],
      ],
      config.style,
    );

    expect(size.width).toBe(248);
  });

  it("uses custom gap item instead of adding default gap around it", () => {
    const config = createDefaultConfig();
    const size = estimateOverlaySize(
      {
        unitPx: 100,
        gapUnit: 0.2,
      },
      [
        [
          { type: "key", id: "a", label: "A", group: "movement", widthUnit: 1 },
          { type: "gap", widthUnit: 0.5 },
          { type: "key", id: "s", label: "S", group: "movement", widthUnit: 1 },
        ],
      ],
      config.style,
    );

    expect(size.width).toBe(278);
  });
});
