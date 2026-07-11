import { describe, expect, it } from "vitest";
import { createDefaultConfig } from "./defaultConfig";
import { estimateOverlaySize } from "./overlaySize";

describe("estimateOverlaySize", () => {
  it("scales default grid overlay dimensions", () => {
    const config = createDefaultConfig();
    const normal = estimateOverlaySize(config.layout, config.keys, config.style);
    const large = estimateOverlaySize(config.layout, config.keys, {
      ...config.style,
      scale: 2,
    });

    expect(large.width).toBeGreaterThan(normal.width);
    expect(large.height).toBeGreaterThan(normal.height);
  });

  it("includes the transparent window padding", () => {
    const config = createDefaultConfig();
    const size = estimateOverlaySize(config.layout, config.keys, config.style);

    expect(size.height).toBeGreaterThan(5 * config.layout.unitPx);
  });

  it("uses row layout width units", () => {
    const config = createDefaultConfig();
    const size = estimateOverlaySize(
      config.layout,
      [
        { id: "a", label: "A", group: "movement", row: 0, widthUnit: 1 },
        { id: "space", label: "Space", group: "modifier", row: 0, widthUnit: 4 },
      ],
      config.style,
    );

    expect(size.width).toBeGreaterThan(config.layout.unitPx * 5);
  });
});
