import { describe, expect, it } from "vitest";
import { buildConfigFileJson, parseConfigFile } from "./configFile";
import { createDefaultConfig } from "./defaultConfig";

describe("parseConfigFile", () => {
  it("loads overlay layout, style, and keys from json text", () => {
    const config = parseConfigFile(
      JSON.stringify({
        version: 1,
        name: "Left keyboard",
        overlay: {
          visible: true,
          position: "bottom-left",
          layout: {
            unitPx: 60,
            gapUnit: 0.155,
          },
          style: {
            scale: 1,
            opacity: 0.92,
            backgroundMode: "transparent",
            backgroundColor: "#0a0c0e",
            backgroundOpacity: 0.72,
            backgroundRadius: 8,
            idleKeyVisibility: "visible",
            alwaysOnTop: false,
            idleColor: "#121417",
            activeColor: "#25d366",
            idleTextColor: "#f5f7fa",
            activeTextColor: "#ffffff",
          },
          keys: [
            {
              id: "w",
              label: "W",
              group: "movement",
              row: 0,
              widthUnit: 1.234,
            },
          ],
        },
        recording: {
          defaultFps: 60,
          fpsOptions: [30, 60, 120],
          customFpsEnabled: true,
          customFps: 1200,
          maxFps: 1000,
          formatExtension: ".kbdrec",
          primaryArtifact: "input-binary",
        },
      }),
    );

    expect(config.name).toBe("Left keyboard");
    expect(config.overlay.layout.unitPx).toBe(60);
    expect(config.overlay.layout.gapUnit).toBe(0.16);
    expect(config.overlay.keys[0]).toMatchObject({
      id: "w",
      label: "W",
      row: 0,
      widthUnit: 1.23,
    });
    expect(config.recording.defaultFps).toBe(60);
    expect(config.recording.customFpsEnabled).toBe(true);
    expect(config.recording.customFps).toBe(1000);
    expect(config.recording.maxFps).toBe(1000);
  });

  it("loads row layout with custom gaps", () => {
    const config = parseConfigFile(
      JSON.stringify({
        version: 1,
        name: "Rows",
        overlay: {
          visible: true,
          position: "bottom-left",
          layout: {
            unitPx: 60,
            gapUnit: 0.15,
          },
          style: createDefaultConfig().style,
          rows: [
            [
              { type: "key", id: "q", label: "Q", group: "action", widthUnit: 1 },
              { type: "gap", widthUnit: 0.5 },
              { type: "key", id: "w", label: "W", group: "movement", widthUnit: 1.234 },
            ],
          ],
        },
      }),
    );

    expect(config.overlay.rows).toEqual([
      [
        { type: "key", id: "q", label: "Q", group: "action", widthUnit: 1 },
        { type: "gap", widthUnit: 0.5 },
        { type: "key", id: "w", label: "W", group: "movement", widthUnit: 1.23 },
      ],
    ]);
    expect(config.overlay.keys.map((key) => key.id)).toEqual(["q", "w"]);
    expect(config.overlay.keys[1].row).toBe(0);
  });
});

describe("buildConfigFileJson", () => {
  it("serializes the current config with name and overlay state", () => {
    const config = createDefaultConfig();
    const json = buildConfigFileJson({
      name: "My profile",
      config,
      visible: false,
      position: "center",
    });
    const parsed = JSON.parse(json);

    expect(parsed.name).toBe("My profile");
    expect(parsed.overlay.visible).toBe(false);
    expect(parsed.overlay.position).toBe("center");
    expect(parsed.overlay.layout).toEqual(config.layout);
    expect(parsed.overlay.style).toEqual(config.style);
    expect(parsed.overlay.rows).toEqual(config.rows);
    expect(parsed.overlay.keys).toBeUndefined();
  });
});
