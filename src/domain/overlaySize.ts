import type { KeyBinding, OverlayLayout, OverlayStyle } from "./defaultConfig";
import { normalizeUnit } from "./layoutUnits";

export type OverlaySize = {
  width: number;
  height: number;
};

const DEFAULT_COLUMN_COUNT = 4;
const DEFAULT_KEYBOARD_ROW_COUNT = 4;
const DEFAULT_MOUSE_ROW_COUNT = 1;
const OVERLAY_WINDOW_PADDING = 14 * 2;

export function estimateOverlaySize(
  layout: OverlayLayout,
  keys: KeyBinding[],
  style: OverlayStyle,
): OverlaySize {
  const unit = layout.unitPx * style.scale;
  const gap = unit * normalizeUnit(layout.gapUnit);
  const padding = style.backgroundMode === "panel" ? 18 * 2 : 0;

  if (keys.some((key) => key.row !== undefined)) {
    const rows = groupRows(keys);
    const widthUnits = Math.max(
      ...rows.map((row) => row.reduce((sum, key) => sum + normalizeUnit(key.widthUnit), 0)),
      1,
    );
    const maxKeysInRow = Math.max(...rows.map((row) => row.length), 1);

    return {
      width: Math.ceil(widthUnits * unit + (maxKeysInRow - 1) * gap + padding + OVERLAY_WINDOW_PADDING),
      height: Math.ceil(rows.length * unit + (rows.length - 1) * gap + padding + OVERLAY_WINDOW_PADDING),
    };
  }

  return {
    width: Math.ceil(
      DEFAULT_COLUMN_COUNT * unit +
        (DEFAULT_COLUMN_COUNT - 1) * gap +
        padding +
        OVERLAY_WINDOW_PADDING,
    ),
    height: Math.ceil(
      (DEFAULT_MOUSE_ROW_COUNT + DEFAULT_KEYBOARD_ROW_COUNT) * unit +
        DEFAULT_KEYBOARD_ROW_COUNT * gap +
        padding +
        OVERLAY_WINDOW_PADDING,
    ),
  };
}

function groupRows(keys: KeyBinding[]): KeyBinding[][] {
  const rowMap = new Map<number, KeyBinding[]>();

  for (const key of keys) {
    const row = key.row ?? 0;
    rowMap.set(row, [...(rowMap.get(row) ?? []), key]);
  }

  return [...rowMap.entries()]
    .sort(([left], [right]) => left - right)
    .map(([, row]) => row);
}
