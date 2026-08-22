// What a right-click on a table cell offers, and what each item does.
//
// The toolbar under the grid can add a row at the bottom and remove the
// selected ones. That is the right set of buttons and the wrong set of
// gestures: a row belongs in the MIDDLE of a table at least as often as at
// the end, and the hand that wants to remove row 12 is already on row 12. So
// the same operations are also where a spreadsheet has always put them —
// under the pointer, on the thing being acted on.
//
// Pure: the items are computed from the selection and the table's size, and
// the caller applies the edit. That is what lets the labels be tested (they
// name the actual rows, which is the part that goes wrong) without a grid.

import { columnLabel } from "./cellRef";
import type { ContextMenuItem } from "../components/ContextMenu";

/** Every action the menu can ask for. */
export type TableMenuAction =
  | "insert-rows-above"
  | "insert-rows-below"
  | "insert-columns-left"
  | "insert-columns-right"
  | "delete-rows"
  | "delete-columns";

/** What the menu needs to know about where the click landed. */
export interface TableMenuContext {
  /** Rows the selection covers, ascending. Empty means nothing is selected,
   * which the caller avoids by selecting the clicked cell first. */
  rows: readonly number[];
  /** Columns the selection covers, ascending. */
  columns: readonly number[];
  /** The table's size, which is what makes "delete every row" refusable. */
  height: number;
  width: number;
}

/** "row 3" / "rows 3–5", in the numbering a person reads off the grid. */
export function rowsPhrase(rows: readonly number[]): string {
  if (rows.length === 0) return "no rows";
  if (rows.length === 1) return `row ${rows[0] + 1}`;
  return `rows ${rows[0] + 1}–${rows[rows.length - 1] + 1}`;
}

/** "column B" / "columns B–D". */
export function columnsPhrase(columns: readonly number[]): string {
  if (columns.length === 0) return "no columns";
  if (columns.length === 1) return `column ${columnLabel(columns[0])}`;
  return `columns ${columnLabel(columns[0])}–${columnLabel(columns[columns.length - 1])}`;
}

/** "a row" / "3 rows" — the count an insert is about to add. */
function count(n: number, singular: string): string {
  return n === 1 ? `a ${singular}` : `${n} ${singular}s`;
}

/**
 * The menu for this selection.
 *
 * Removing every row (or every column) is offered and DISABLED rather than
 * hidden, with the tooltip saying why: an item that disappears when you
 * select the whole table teaches nobody that the whole table was the problem.
 */
export function tableMenuItems(at: TableMenuContext): ContextMenuItem[] {
  const rows = at.rows.length;
  const columns = at.columns.length;
  const allRows = rows >= at.height;
  const allColumns = columns >= at.width;
  return [
    {
      id: "insert-rows-above",
      label: `Insert ${count(rows, "row")} above`,
      tip: `Add ${count(rows, "row")} above ${rowsPhrase(at.rows)}`,
    },
    {
      id: "insert-rows-below",
      label: `Insert ${count(rows, "row")} below`,
      tip: `Add ${count(rows, "row")} below ${rowsPhrase(at.rows)}`,
    },
    {
      id: "insert-columns-left",
      label: `Insert ${count(columns, "column")} left`,
      group: true,
      tip: `Add ${count(columns, "column")} to the left of ${columnsPhrase(at.columns)}`,
    },
    {
      id: "insert-columns-right",
      label: `Insert ${count(columns, "column")} right`,
      tip: `Add ${count(columns, "column")} to the right of ${columnsPhrase(at.columns)}`,
    },
    {
      id: "delete-rows",
      label: rows > 1 ? `Delete ${rows} rows` : "Delete row",
      group: true,
      danger: true,
      disabled: rows === 0 || allRows,
      tip: allRows
        ? "That is every row — a table with none left is not a table"
        : `Remove ${rowsPhrase(at.rows)}`,
    },
    {
      id: "delete-columns",
      label: columns > 1 ? `Delete ${columns} columns` : "Delete column",
      danger: true,
      disabled: columns === 0 || allColumns,
      tip: allColumns
        ? "That is every column — a table with none left is not a table"
        : `Remove ${columnsPhrase(at.columns)}`,
    },
  ];
}
