import {
  createDefaultConfig,
  flattenRowKeys,
  isKeyBinding,
  type AppConfig,
  type KeyBinding,
  type OverlayLayout,
  type OverlayRow,
  type OverlayRowItem,
  type OverlayStyle,
} from "./defaultConfig";
import { normalizeUnit } from "./layoutUnits";
import { normalizeRecordingConfig } from "./recordingConfig";

export type OverlayConfigFile = {
  version: number;
  name?: string;
  overlay: {
    visible?: boolean;
    position?: string;
    customPosition?: {
      x: number;
      y: number;
    } | null;
    layout: OverlayLayout;
    style: OverlayStyle;
    rows: OverlayRow[];
    keys: KeyBinding[];
  };
  recording: AppConfig["recording"];
  export: AppConfig["export"];
};

export function parseConfigFile(text: string): OverlayConfigFile {
  const config = JSON.parse(text) as OverlayConfigFile;
  const defaultConfig = createDefaultConfig();
  const rows = normalizeRows(config.overlay.rows ?? rowsFromLegacyKeys(config.overlay.keys ?? []));
  const recording = normalizeRecordingConfig(config.recording ?? defaultConfig.recording);
  const exportConfig = normalizeExportConfig(config.export ?? defaultConfig.export);

  return {
    ...config,
    overlay: {
      ...config.overlay,
      layout: {
        ...config.overlay.layout,
        gapUnit: normalizeUnit(config.overlay.layout.gapUnit),
      },
      rows,
      keys: flattenRowKeys(rows),
    },
    recording,
    export: exportConfig,
  };
}

export function buildConfigFileJson({
  name,
  config,
  visible,
  position,
  customPosition = null,
}: {
  name: string;
  config: AppConfig;
  visible: boolean;
  position: string;
  customPosition?: { x: number; y: number } | null;
}): string {
  return `${JSON.stringify(
    {
      version: 1,
      name,
      overlay: {
        visible,
        position,
        customPosition,
        layout: config.layout,
        style: config.style,
        rows: config.rows,
      },
      recording: config.recording,
      export: config.export,
    },
    null,
    2,
  )}\n`;
}

function normalizeExportConfig(exportConfig: Partial<AppConfig["export"]>): AppConfig["export"] {
  const defaultExport = createDefaultConfig().export;

  return {
    defaultFormat: exportConfig.defaultFormat ?? defaultExport.defaultFormat,
    transparentFormat: exportConfig.transparentFormat ?? defaultExport.transparentFormat,
    compatibleFormat: exportConfig.compatibleFormat ?? defaultExport.compatibleFormat,
    renderMarkers: exportConfig.renderMarkers ?? true,
  };
}

function normalizeRows(rows: OverlayRow[]): OverlayRow[] {
  return rows.map((row) =>
    row.map((item): OverlayRowItem => {
      if (!isKeyBinding(item)) {
        return {
          ...item,
          widthUnit: normalizeUnit(item.widthUnit),
        };
      }

      return {
        ...item,
        type: "key",
        widthUnit: normalizeUnit(item.widthUnit),
      };
    }),
  );
}

function rowsFromLegacyKeys(keys: KeyBinding[]): OverlayRow[] {
  const rowMap = new Map<number, KeyBinding[]>();

  for (const key of keys) {
    const row = key.row ?? 0;
    rowMap.set(row, [
      ...(rowMap.get(row) ?? []),
      {
        ...key,
        type: "key",
        widthUnit: normalizeUnit(key.widthUnit),
      },
    ]);
  }

  return [...rowMap.entries()]
    .sort(([left], [right]) => left - right)
    .map(([, row]) => row);
}
