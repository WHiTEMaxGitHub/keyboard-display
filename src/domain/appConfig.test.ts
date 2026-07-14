import { describe, expect, it } from "vitest";
import { buildAppConfigFile, parseAppConfigFile } from "./appConfig";
import { createDefaultConfig } from "./defaultConfig";

describe("app config", () => {
  it("builds an app config with current profile state", () => {
    const config = createDefaultConfig();
    const appConfig = buildAppConfigFile({
      defaultProfilePath: "docs/default-config.json",
      currentProfile: {
        name: "CS POV",
        sourcePath: "/tmp/cs-pov.json",
        dirty: true,
        overlay: {
          visible: true,
          position: "bottom-right",
          layout: config.layout,
          style: config.style,
          keys: config.keys,
        },
      },
      recording: {
        outputDirectory: "/tmp/recordings",
        hotkeys: {
          mode: "toggle",
          start: ["ctrl-left", "r"],
          stop: ["ctrl-left", "r"],
        },
      },
    });

    expect(appConfig.profiles.lastProfilePath).toBe("/tmp/cs-pov.json");
    expect(appConfig.profiles.recentProfiles).toEqual([
      { name: "CS POV", path: "/tmp/cs-pov.json" },
    ]);
    expect(appConfig.currentProfile.overlay.position).toBe("bottom-right");
    expect(appConfig.recording.outputDirectory).toBe("/tmp/recordings");
  });

  it("parses app config json", () => {
    const appConfig = parseAppConfigFile(
      JSON.stringify({
        version: 1,
        profiles: {
          defaultProfilePath: "docs/default-config.json",
          lastProfilePath: null,
          recentProfiles: [],
        },
        currentProfile: {
          name: "Unsaved",
          sourcePath: null,
          dirty: true,
          overlay: {
            visible: true,
            position: "center",
            layout: createDefaultConfig().layout,
            style: createDefaultConfig().style,
            keys: createDefaultConfig().keys,
          },
        },
        recording: {
          outputDirectory: null,
          hotkeys: {
            mode: "disabled",
            start: [],
            stop: [],
          },
        },
        ui: {
          language: "system",
        },
      }),
    );

    expect(appConfig.currentProfile.name).toBe("Unsaved");
    expect(appConfig.currentProfile.overlay.position).toBe("center");
  });
});
