import {
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

export type OverlayConfigFile = {
  version: number;
  name?: string;
  overlay: {
    visible?: boolean;
    position?: string;
    layout: OverlayLayout;
    style: OverlayStyle;
    rows: OverlayRow[];
    keys: KeyBinding[];
  };
};

export function parseConfigFile(text: string): OverlayConfigFile {
  const config = JSON.parse(text) as OverlayConfigFile;
  const rows = normalizeRows(config.overlay.rows ?? rowsFromLegacyKeys(config.overlay.keys ?? []));

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
        rows: config.rows,
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
