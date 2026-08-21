// Protects docs/guarantees/authoring/a-table-is-a-dataset-and-a-paragraph.md
// — "selection is a rectangle, not a cursor".

import { describe, expect, it } from "vitest";

import {
  allSelection,
  boundsOf,
  cellSelection,
  cellsIn,
  columnSelection,
  columnsIn,
  containsCell,
  extendTo,
  isSingleCell,
  rowSelection,
  rowsIn,
  selectionLabel,
  selectionSize,
} from "./tableSelection";

describe("a selection is an anchor and a reach, not two sorted corners", () => {
  it("keeps the anchor still when the reach goes backwards", () => {
    // The anchor is the ACTIVE cell — what the formula bar edits and what a
    // keystroke replaces. Normalising early would throw it away.
    const dragged = extendTo(cellSelection(3, 3), 1, 1, 8, 8);
    expect(dragged.anchor).toEqual({ row: 3, column: 3 });
    expect(boundsOf(dragged)).toEqual({ top: 1, left: 1, bottom: 3, right: 3 });
  });

  it("shrinks rather than flips when a drag comes back over its start", () => {
    const out = extendTo(cellSelection(0, 0), 4, 4, 8, 8);
    const back = extendTo(out, 1, 1, 8, 8);
    expect(selectionSize(back)).toEqual({ rows: 2, columns: 2 });
  });

  it("will not reach past the grid", () => {
    const out = extendTo(cellSelection(0, 0), 99, 99, 3, 2);
    expect(boundsOf(out)).toEqual({ top: 0, left: 0, bottom: 2, right: 1 });
  });
});

describe("whole rows and whole columns", () => {
  it("selects a column down its whole height", () => {
    const out = columnSelection(1, 4);
    expect(out.kind).toBe("columns");
    expect(boundsOf(out)).toEqual({ top: 0, left: 1, bottom: 3, right: 1 });
  });

  it("selects a row across its whole width", () => {
    const out = rowSelection(2, 3);
    expect(out.kind).toBe("rows");
    expect(boundsOf(out)).toEqual({ top: 2, left: 0, bottom: 2, right: 2 });
  });

  it("drags across the letters and never selects half a column", () => {
    // Dragging B → D is three whole columns. A partial one would be a
    // rectangle nobody asked for.
    const out = extendTo(columnSelection(1, 4), 2, 3, 4, 5);
    expect(boundsOf(out)).toEqual({ top: 0, left: 1, bottom: 3, right: 3 });
    expect(columnsIn(out)).toEqual([1, 2, 3]);
  });

  it("drags down the numbers and never selects half a row", () => {
    const out = extendTo(rowSelection(0, 3), 2, 1, 4, 3);
    expect(boundsOf(out)).toEqual({ top: 0, left: 0, bottom: 2, right: 2 });
    expect(rowsIn(out)).toEqual([0, 1, 2]);
  });

  it("selects everything from the corner", () => {
    expect(boundsOf(allSelection(3, 2))).toEqual({ top: 0, left: 0, bottom: 2, right: 1 });
  });
});

describe("what is in it", () => {
  it("knows a single cell from a range", () => {
    expect(isSingleCell(cellSelection(1, 1))).toBe(true);
    expect(isSingleCell(extendTo(cellSelection(1, 1), 1, 2, 4, 4))).toBe(false);
  });

  it("contains every cell in the rectangle and nothing outside it", () => {
    const out = extendTo(cellSelection(1, 1), 2, 2, 5, 5);
    expect(containsCell(out, 2, 1)).toBe(true);
    expect(containsCell(out, 0, 1)).toBe(false);
    expect(containsCell(out, 2, 3)).toBe(false);
  });

  it("lists its cells in reading order", () => {
    expect(cellsIn(extendTo(cellSelection(0, 0), 1, 1, 4, 4))).toEqual([
      { row: 0, column: 0 },
      { row: 0, column: 1 },
      { row: 1, column: 0 },
      { row: 1, column: 1 },
    ]);
  });
});

describe("what the name box says", () => {
  it("names one cell in A1 notation", () => {
    expect(selectionLabel(cellSelection(2, 1))).toBe("B3");
  });

  it("names a range by its corners", () => {
    expect(selectionLabel(extendTo(cellSelection(0, 0), 2, 1, 4, 4))).toBe("A1:B3");
  });

  it("names a whole column without claiming how tall it is", () => {
    // `B1:B7` would stop being true the moment somebody adds a row.
    expect(selectionLabel(columnSelection(1, 7))).toBe("B:B");
    expect(selectionLabel(extendTo(columnSelection(1, 7), 0, 3, 7, 5))).toBe("B:D");
  });

  it("names whole rows by their numbers", () => {
    expect(selectionLabel(extendTo(rowSelection(0, 3), 2, 0, 4, 3))).toBe("1:3");
  });
});
