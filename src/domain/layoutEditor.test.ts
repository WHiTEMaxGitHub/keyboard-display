import { describe, expect, it } from "vitest";
import {
  addRow,
  addGapToRow,
  addKeyToRow,
  moveRow,
  moveRowItem,
  removeRow,
  removeRowItem,
  updateRowItem,
} from "./layoutEditor";
import type { OverlayRow } from "./defaultConfig";

describe("layout editor helpers", () => {
  it("adds key and gap items to a row", () => {
    const rows: OverlayRow[] = [[]];

    expect(addKeyToRow(rows, 0)[0][0]).toMatchObject({
      type: "key",
      id: "new-key",
      label: "Key",
      group: "action",
      widthUnit: 1,
    });
    expect(addGapToRow(rows, 0)[0][0]).toEqual({
      type: "gap",
      widthUnit: 1,
    });
  });

  it("updates a row item without mutating the original rows", () => {
    const rows: OverlayRow[] = [[{ type: "gap", widthUnit: 1 }]];
    const updatedRows = updateRowItem(rows, 0, 0, { type: "gap", widthUnit: 2 });

    expect(updatedRows[0][0]).toEqual({ type: "gap", widthUnit: 2 });
    expect(rows[0][0]).toEqual({ type: "gap", widthUnit: 1 });
  });

  it("removes a row item", () => {
    const rows: OverlayRow[] = [[
      { type: "gap", widthUnit: 1 },
      { type: "key", id: "w", label: "W", group: "movement", widthUnit: 1 },
    ]];

    expect(removeRowItem(rows, 0, 0)).toEqual([[
      { type: "key", id: "w", label: "W", group: "movement", widthUnit: 1 },
    ]]);
  });

  it("adds and removes rows", () => {
    const rows: OverlayRow[] = [[{ type: "gap", widthUnit: 1 }]];

    expect(addRow(rows)).toEqual([[{ type: "gap", widthUnit: 1 }], []]);
    expect(removeRow(addRow(rows), 0)).toEqual([[]]);
  });

  it("moves rows and items", () => {
    const rows: OverlayRow[] = [
      [{ type: "key", id: "a", label: "A", group: "movement", widthUnit: 1 }],
      [
        { type: "key", id: "b", label: "B", group: "movement", widthUnit: 1 },
        { type: "gap", widthUnit: 1 },
      ],
    ];

    expect(moveRow(rows, 0, 1).map((row) => row[0])).toEqual([
      { type: "key", id: "b", label: "B", group: "movement", widthUnit: 1 },
      { type: "key", id: "a", label: "A", group: "movement", widthUnit: 1 },
    ]);
    expect(moveRowItem(rows, 1, 0, 1)[1]).toEqual([
      { type: "gap", widthUnit: 1 },
      { type: "key", id: "b", label: "B", group: "movement", widthUnit: 1 },
    ]);
  });
});
