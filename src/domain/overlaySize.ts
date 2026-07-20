import { isKeyBinding, type OverlayLayout, type OverlayRow, type OverlayStyle } from "./defaultConfig";
import { normalizeUnit } from "./layoutUnits";

export type OverlaySize = {
  width: number;
  height: number;
};

const BACKPLATE_PADDING = 10 * 2;
const OVERLAY_BLEED = 12 * 2;
const FLOAT_EPSILON = 0.000001;

export function estimateOverlaySize(
  layout: OverlayLayout,
  rows: OverlayRow[],
  style: OverlayStyle,
): OverlaySize {
  const unit = layout.unitPx * style.scale;
  const gap = unit * normalizeUnit(layout.gapUnit);
  const padding = isBackplateVisible(style.backgroundColor) ? BACKPLATE_PADDING : 0;
  const widthUnits = Math.max(...rows.map((row) => rowWidthUnits(row, layout.gapUnit)), 1);
  const rowCount = Math.max(rows.length, 1);

  return {
    width: ceilStable(widthUnits * unit + padding + OVERLAY_BLEED),
    height: ceilStable(rowCount * unit + (rowCount - 1) * gap + padding + OVERLAY_BLEED),
  };
}

function isBackplateVisible(backgroundColor: string): boolean {
  return !/^#[0-9a-fA-F]{8}$/.test(backgroundColor) || !backgroundColor.endsWith("00");
}

function ceilStable(value: number): number {
  return Math.ceil(value - FLOAT_EPSILON);
}

function rowWidthUnits(row: OverlayRow, gapUnit: number): number {
  return row.reduce((sum, item, index) => {
    const itemWidth = normalizeUnit(item.widthUnit);
    const defaultGap =
      index > 0 && isKeyBinding(item) && isKeyBinding(row[index - 1])
        ? normalizeUnit(gapUnit)
        : 0;

    return sum + itemWidth + defaultGap;
  }, 0);
}
