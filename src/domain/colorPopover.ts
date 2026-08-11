export type ColorPopoverDirection = "down" | "up";

export type ColorPopoverRect = {
  left: number;
  right: number;
  top: number;
  bottom: number;
  width: number;
};

export type ColorPopoverViewport = {
  width: number;
  height: number;
};

export type ColorPopoverPlacement = {
  direction: ColorPopoverDirection;
  left: number;
  top: number;
  width: number;
  maxHeight: number;
};

export type ColorPopoverPlacementOptions = {
  gap?: number;
  margin?: number;
  maxHeight?: number;
  minHeight?: number;
  minWidth?: number;
  panelHeight?: number;
};

const DEFAULT_GAP = 8;
const DEFAULT_MARGIN = 16;
const DEFAULT_MAX_HEIGHT = 360;
const DEFAULT_MIN_HEIGHT = 120;
const DEFAULT_MIN_WIDTH = 260;

export function placeColorPopover(
  triggerRect: ColorPopoverRect,
  viewport: ColorPopoverViewport,
  options: ColorPopoverPlacementOptions = {},
): ColorPopoverPlacement {
  const gap = options.gap ?? DEFAULT_GAP;
  const margin = options.margin ?? DEFAULT_MARGIN;
  const maxHeight = options.maxHeight ?? DEFAULT_MAX_HEIGHT;
  const minHeight = options.minHeight ?? DEFAULT_MIN_HEIGHT;
  const minWidth = options.minWidth ?? DEFAULT_MIN_WIDTH;
  const availableWidth = Math.max(0, viewport.width - margin * 2);
  const width = Math.min(Math.max(triggerRect.width, minWidth), availableWidth);
  const left = clamp(triggerRect.left, margin, viewport.width - margin - width);
  const preferredHeight = Math.min(options.panelHeight ?? maxHeight, maxHeight);
  const spaceBelow = viewport.height - triggerRect.bottom - margin - gap;
  const spaceAbove = triggerRect.top - margin - gap;
  const direction = spaceBelow < preferredHeight && spaceAbove > spaceBelow ? "up" : "down";
  const availableHeight = Math.max(0, direction === "down" ? spaceBelow : spaceAbove);
  const resolvedMaxHeight = Math.max(minHeight, Math.min(maxHeight, availableHeight));
  const heightForPlacement = Math.min(preferredHeight, resolvedMaxHeight);
  const rawTop = direction === "down"
    ? triggerRect.bottom + gap
    : triggerRect.top - gap - heightForPlacement;

  return {
    direction,
    left,
    top: clamp(rawTop, margin, viewport.height - margin - heightForPlacement),
    width,
    maxHeight: resolvedMaxHeight,
  };
}

function clamp(value: number, min: number, max: number) {
  if (max < min) {
    return min;
  }

  return Math.min(max, Math.max(min, value));
}
