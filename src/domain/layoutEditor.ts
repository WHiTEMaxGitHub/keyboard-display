import type { OverlayRow, OverlayRowItem } from "./defaultConfig";

export function addKeyToRow(rows: OverlayRow[], rowIndex: number): OverlayRow[] {
  return updateRows(rows, rowIndex, (row) => [
    ...row,
    {
      type: "key",
      id: "new-key",
      label: "Key",
      group: "action",
      widthUnit: 1,
    },
  ]);
}

export function addGapToRow(rows: OverlayRow[], rowIndex: number): OverlayRow[] {
  return updateRows(rows, rowIndex, (row) => [
    ...row,
    {
      type: "gap",
      widthUnit: 1,
    },
  ]);
}

export function updateRowItem(
  rows: OverlayRow[],
  rowIndex: number,
  itemIndex: number,
  item: OverlayRowItem,
): OverlayRow[] {
  return updateRows(rows, rowIndex, (row) =>
    row.map((currentItem, currentIndex) =>
      currentIndex === itemIndex ? { ...item } : { ...currentItem },
    ),
  );
}

export function removeRowItem(
  rows: OverlayRow[],
  rowIndex: number,
  itemIndex: number,
): OverlayRow[] {
  return updateRows(rows, rowIndex, (row) =>
    row.filter((_, currentIndex) => currentIndex !== itemIndex).map((item) => ({ ...item })),
  );
}

function updateRows(
  rows: OverlayRow[],
  rowIndex: number,
  updateRow: (row: OverlayRow) => OverlayRow,
): OverlayRow[] {
  return rows.map((row, currentIndex) =>
    currentIndex === rowIndex
      ? updateRow(row.map((item) => ({ ...item })))
      : row.map((item) => ({ ...item })),
  );
}
