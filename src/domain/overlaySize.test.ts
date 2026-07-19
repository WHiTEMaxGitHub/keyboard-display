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

  it("keeps a stable minimum size for empty row layouts", () => {
    const config = createDefaultConfig();
    const size = estimateOverlaySize(config.layout, [], config.style);

    expect(size.width).toBeGreaterThan(0);
    expect(size.height).toBeGreaterThan(0);
  });

  it("does not include panel padding when the backplate is transparent", () => {
    const config = createDefaultConfig();
    const panelSize = estimateOverlaySize(config.layout, config.rows, {
      ...config.style,
      backgroundMode: "panel",
      backgroundColor: "#0a0c0e00",
    });
    const transparentSize = estimateOverlaySize(config.layout, config.rows, {
      ...config.style,
      backgroundMode: "transparent",
      backgroundColor: "#0a0c0e00",
    });

    expect(panelSize.width - transparentSize.width).toBe(36);
    expect(panelSize.height - transparentSize.height).toBe(36);
  });
});
