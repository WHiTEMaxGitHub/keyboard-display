import { describe, expect, it } from "vitest";
import { createDefaultConfig } from "./defaultConfig";

describe("createDefaultConfig", () => {
  it("describes the desktop overlay and config surfaces", () => {
    const config = createDefaultConfig();

    expect(config.surfaces).toEqual([
      {
        id: "pov",
        title: "POV Overlay",
        role: "overlay",
      },
      {
        id: "config",
        title: "Control Panel",
        role: "settings",
      },
    ]);
  });

  it("uses a CS-oriented key layout", () => {
    const config = createDefaultConfig();
    const keyIds = config.keys.map((key) => key.id);

    expect(keyIds).toEqual([
      "mouse-left",
      "mouse-right",
      "w",
      "r",
      "a",
      "s",
      "d",
      "q",
      "shift-left",
      "ctrl-left",
      "e",
      "space",
    ]);

    expect(config.layout.unitPx).toBe(54);
    expect(config.layout.gapUnit).toBe(0.15);
    expect(config.rows).toHaveLength(5);
    expect(config.rows[1][0]).toMatchObject({
      type: "gap",
      widthUnit: 1,
    });
    expect(config.keys.find((key) => key.id === "w")?.widthUnit).toBe(1);
    expect(config.keys.find((key) => key.id === "space")?.widthUnit).toBe(4);
    expect(config.keys.find((key) => key.id === "mouse-left")?.widthUnit).toBe(1.35);
    expect(JSON.stringify(config.keys)).not.toContain("gridArea");
  });

  it("stores recording as input frames instead of rendered video", () => {
    const config = createDefaultConfig();

    expect(config.recording.defaultFps).toBe(60);
    expect(config.recording.fpsOptions).toEqual([30, 60, 120]);
    expect(config.recording.formatExtension).toBe(".kbdrec");
    expect(config.recording.primaryArtifact).toBe("input-binary");
  });

  it("defaults to a transparent overlay with visible idle keys", () => {
    const config = createDefaultConfig();

    expect(config.style.backgroundMode).toBe("transparent");
    expect(config.style.backgroundColor).toBe("#0a0c0e");
    expect(config.style.backgroundOpacity).toBe(0.72);
    expect(config.style.backgroundRadius).toBe(8);
    expect(config.style.idleKeyVisibility).toBe("visible");
    expect(config.style.alwaysOnTop).toBe(false);
    expect(config.style.idleTextColor).toBe("#f5f7fa");
    expect(config.style.activeTextColor).toBe("#ffffff");
  });
});
