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
    expect(parsed.overlay.keys).toEqual(config.keys);
  });
});
