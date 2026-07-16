import { isKeyBinding, type OverlayLayout, type OverlayRow, type OverlayStyle } from "./defaultConfig";
import { normalizeUnit } from "./layoutUnits";

export type OverlaySize = {
  width: number;
  height: number;
};

const OVERLAY_WINDOW_PADDING = 14 * 2;
const FLOAT_EPSILON = 0.000001;

export function estimateOverlaySize(
  layout: OverlayLayout,
  rows: OverlayRow[],
  style: OverlayStyle,
): OverlaySize {
  const unit = layout.unitPx * style.scale;
  const gap = unit * normalizeUnit(layout.gapUnit);
  const padding = style.backgroundMode === "panel" ? 18 * 2 : 0;
  const widthUnits = Math.max(...rows.map((row) => rowWidthUnits(row, layout.gapUnit)), 1);

  return {
    width: ceilStable(widthUnits * unit + padding + OVERLAY_WINDOW_PADDING),
    height: ceilStable(rows.length * unit + (rows.length - 1) * gap + padding + OVERLAY_WINDOW_PADDING),
  };
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
