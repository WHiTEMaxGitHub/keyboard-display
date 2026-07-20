import type { OverlayRow, OverlayRowItem } from "./defaultConfig";

export function addRow(rows: OverlayRow[]): OverlayRow[] {
  return [...cloneRows(rows), []];
}

export function removeRow(rows: OverlayRow[], rowIndex: number): OverlayRow[] {
  return cloneRows(rows).filter((_, currentIndex) => currentIndex !== rowIndex);
}

export function moveRow(rows: OverlayRow[], fromIndex: number, toIndex: number): OverlayRow[] {
  return moveArrayItem(cloneRows(rows), fromIndex, toIndex);
}

export function moveRowItem(
  rows: OverlayRow[],
  rowIndex: number,
  fromIndex: number,
  toIndex: number,
): OverlayRow[] {
  return updateRows(rows, rowIndex, (row) => moveArrayItem(row, fromIndex, toIndex));
}

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
  return cloneRows(rows).map((row, currentIndex) =>
    currentIndex === rowIndex
      ? updateRow(row)
      : row,
  );
}

function cloneRows(rows: OverlayRow[]): OverlayRow[] {
  return rows.map((row) => row.map((item) => ({ ...item })));
}

function moveArrayItem<T>(items: T[], fromIndex: number, toIndex: number): T[] {
  const nextItems = [...items];
  if (
    fromIndex < 0 ||
    fromIndex >= nextItems.length ||
    toIndex < 0 ||
    toIndex >= nextItems.length ||
    fromIndex === toIndex
  ) {
    return nextItems;
  }

  const [item] = nextItems.splice(fromIndex, 1);
  nextItems.splice(toIndex, 0, item);
  return nextItems;
}
