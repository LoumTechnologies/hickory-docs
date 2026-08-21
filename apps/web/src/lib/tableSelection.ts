// What is selected in a table, as a range rather than a cursor.
//
// A spreadsheet's selection is a rectangle with a corner you started from,
// and that corner matters: it is the cell the formula bar edits, the cell a
// keystroke replaces, and the cell an extend keeps still. So this is an
// ANCHOR and a FOCUS, not a pair of sorted corners — normalising early would
// throw away the only part of a drag that says which direction it went.
//
// The `kind` is here for the same reason. A column selection and a
// cell selection that happens to cover a whole column look identical as
// rectangles and are not the same thing: one says "column B", and asking to
// remove a row from it should be refused rather than quietly emptying the
// table. Keeping what the person actually clicked is cheaper than inferring
// it back from the shape.
//
// Everything here is pure and takes the grid's measurements as arguments —
// the panel owns the state, this owns the arithmetic.

import { cellLabel, columnLabel } from "./cellRef";

export interface Cell {
  row: number;
  column: number;
}

/** What was clicked to make this selection: cells, whole rows, or whole
 * columns. Not a rendering detail — see the note above. */
export type SelectionKind = "cells" | "rows" | "columns";

export interface Selection {
  /** Where the selection started. The ACTIVE cell: what the formula bar
   * edits and what a keystroke replaces. */
  anchor: Cell;
  /** Where it currently reaches. Equal to the anchor for a single cell. */
  focus: Cell;
  kind: SelectionKind;
}

/** A normalised rectangle, inclusive on every side. */
export interface Bounds {
  top: number;
  left: number;
  bottom: number;
  right: number;
}

const clamp = (value: number, limit: number) => Math.max(0, Math.min(value, limit - 1));

/** One cell, selected on its own. */
export function cellSelection(row: number, column: number): Selection {
  const at = { row, column };
  return { anchor: at, focus: at, kind: "cells" };
}

/** A whole column, from its letter. */
export function columnSelection(column: number, height: number): Selection {
  return {
    anchor: { row: 0, column },
    focus: { row: Math.max(0, height - 1), column },
    kind: "columns",
  };
}

/** A whole row, from its number. */
export function rowSelection(row: number, width: number): Selection {
  return {
    anchor: { row, column: 0 },
    focus: { row, column: Math.max(0, width - 1) },
    kind: "rows",
  };
}

/** Everything — the corner box, and Ctrl+A. */
export function allSelection(height: number, width: number): Selection {
  return {
    anchor: { row: 0, column: 0 },
    focus: { row: Math.max(0, height - 1), column: Math.max(0, width - 1) },
    kind: "cells",
  };
}

/**
 * The same selection reaching to another cell — a drag, or a shift-click.
 *
 * The anchor never moves, which is what makes dragging back over the start
 * shrink the range instead of dropping it. A row or column selection extends
 * along its own axis and stays full along the other: dragging across the
 * letters B, C, D selects three whole columns, never a partial one.
 */
export function extendTo(
  selection: Selection,
  row: number,
  column: number,
  height: number,
  width: number,
): Selection {
  const at = { row: clamp(row, height), column: clamp(column, width) };
  if (selection.kind === "columns") {
    return {
      ...selection,
      anchor: { row: 0, column: selection.anchor.column },
      focus: { row: Math.max(0, height - 1), column: at.column },
    };
  }
  if (selection.kind === "rows") {
    return {
      ...selection,
      anchor: { row: selection.anchor.row, column: 0 },
      focus: { row: at.row, column: Math.max(0, width - 1) },
    };
  }
  return { ...selection, focus: at };
}

/** The rectangle, with its corners in reading order. */
export function boundsOf(selection: Selection): Bounds {
  return {
    top: Math.min(selection.anchor.row, selection.focus.row),
    bottom: Math.max(selection.anchor.row, selection.focus.row),
    left: Math.min(selection.anchor.column, selection.focus.column),
    right: Math.max(selection.anchor.column, selection.focus.column),
  };
}

export function containsCell(selection: Selection, row: number, column: number): boolean {
  const at = boundsOf(selection);
  return row >= at.top && row <= at.bottom && column >= at.left && column <= at.right;
}

/** Whether this is one cell — the state a spreadsheet spends most of its time
 * in, and the one where a range highlight would be noise. */
export function isSingleCell(selection: Selection): boolean {
  return (
    selection.anchor.row === selection.focus.row &&
    selection.anchor.column === selection.focus.column
  );
}

export function rowsIn(selection: Selection): number[] {
  const at = boundsOf(selection);
  return Array.from({ length: at.bottom - at.top + 1 }, (_, i) => at.top + i);
}

export function columnsIn(selection: Selection): number[] {
  const at = boundsOf(selection);
  return Array.from({ length: at.right - at.left + 1 }, (_, i) => at.left + i);
}

/** Every cell in the rectangle, in reading order. */
export function cellsIn(selection: Selection): Cell[] {
  const cells: Cell[] = [];
  for (const row of rowsIn(selection)) {
    for (const column of columnsIn(selection)) cells.push({ row, column });
  }
  return cells;
}

/** How big it is, for the "3R × 2C" a spreadsheet shows while you drag. */
export function selectionSize(selection: Selection): { rows: number; columns: number } {
  const at = boundsOf(selection);
  return { rows: at.bottom - at.top + 1, columns: at.right - at.left + 1 };
}

/**
 * What the name box says.
 *
 * A1 range notation, because it is what a spreadsheet's name box says and
 * what the person will type back. A whole column is `B:B` rather than
 * `B1:B7` — the bounded spelling would claim the selection is seven rows
 * tall, which stops being true the moment a row is added.
 */
export function selectionLabel(selection: Selection): string {
  const at = boundsOf(selection);
  if (isSingleCell(selection)) return cellLabel(at.left, at.top);
  if (selection.kind === "columns") {
    return `${columnLabel(at.left)}:${columnLabel(at.right)}`;
  }
  if (selection.kind === "rows") return `${at.top + 1}:${at.bottom + 1}`;
  return `${cellLabel(at.left, at.top)}:${cellLabel(at.right, at.bottom)}`;
}
