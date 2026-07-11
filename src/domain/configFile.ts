import type { AppConfig, KeyBinding, OverlayLayout, OverlayStyle } from "./defaultConfig";
import { normalizeUnit } from "./layoutUnits";

export type OverlayConfigFile = {
  version: number;
  name?: string;
  overlay: {
    visible?: boolean;
    position?: string;
    layout: OverlayLayout;
    style: OverlayStyle;
    keys: KeyBinding[];
  };
};

export function parseConfigFile(text: string): OverlayConfigFile {
  const config = JSON.parse(text) as OverlayConfigFile;

  return {
    ...config,
    overlay: {
      ...config.overlay,
      layout: {
        ...config.overlay.layout,
        gapUnit: normalizeUnit(config.overlay.layout.gapUnit),
      },
      keys: config.overlay.keys.map((key) => ({
        ...key,
        widthUnit: normalizeUnit(key.widthUnit),
      })),
    },
  };
}

export function buildConfigFileJson({
  name,
  config,
  visible,
  position,
}: {
  name: string;
  config: AppConfig;
  visible: boolean;
  position: string;
}): string {
  return `${JSON.stringify(
    {
      version: 1,
      name,
      overlay: {
        visible,
        position,
        layout: config.layout,
        style: config.style,
        keys: config.keys,
      },
      recording: config.recording,
      export: {
        defaultFormat: "webm",
        transparentFormat: "webm",
        compatibleFormat: "mp4",
      },
    },
    null,
    2,
  )}\n`;
}
