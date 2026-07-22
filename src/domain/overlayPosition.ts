export type LogicalRect = {
  x: number;
  y: number;
  width: number;
  height: number;
};

export type OverlayCustomPosition = {
  x: number;
  y: number;
  workArea?: LogicalRect | null;
  scaleFactor?: number | null;
};

export type OverlaySize = {
  width: number;
  height: number;
};

/// 将旧显示器上的自定义位置按 workArea 比例映射到当前显示器，并夹到可见区域。
export function resolveCustomOverlayPosition(
  savedPosition: OverlayCustomPosition,
  currentWorkArea: LogicalRect,
  overlaySize: OverlaySize,
): { x: number; y: number } {
  const savedWorkArea = savedPosition.workArea;
  const mappedPosition = savedWorkArea && savedWorkArea.width > 0 && savedWorkArea.height > 0
    ? mapBetweenWorkAreas(savedPosition, savedWorkArea, currentWorkArea)
    : {
        x: savedPosition.x,
        y: savedPosition.y,
      };

  return clampToWorkArea(mappedPosition, currentWorkArea, overlaySize);
}

export function monitorWorkAreaToRect(
  position: { x: number; y: number },
  size: { width: number; height: number },
): LogicalRect {
  return {
    x: roundCoordinate(position.x),
    y: roundCoordinate(position.y),
    width: roundCoordinate(size.width),
    height: roundCoordinate(size.height),
  };
}

function mapBetweenWorkAreas(
  position: { x: number; y: number },
  from: LogicalRect,
  to: LogicalRect,
) {
  const xRatio = (position.x - from.x) / from.width;
  const yRatio = (position.y - from.y) / from.height;

  return {
    x: to.x + xRatio * to.width,
    y: to.y + yRatio * to.height,
  };
}

function clampToWorkArea(
  position: { x: number; y: number },
  workArea: LogicalRect,
  overlaySize: OverlaySize,
) {
  const maxX = Math.max(workArea.x, workArea.x + workArea.width - overlaySize.width);
  const maxY = Math.max(workArea.y, workArea.y + workArea.height - overlaySize.height);

  return {
    x: roundCoordinate(clamp(position.x, workArea.x, maxX)),
    y: roundCoordinate(clamp(position.y, workArea.y, maxY)),
  };
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

function roundCoordinate(value: number) {
  return Math.round(value);
}
