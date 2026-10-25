<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/TableSizeDialog.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Table Is A Dataset And A Paragraph](../../../../../contract.md)

Captured SHA-256: `46b1235c478dc5a95d9b4688c3139ab364cc98d4d36e03caad938ccb0fe323e4`.

<book:entry path="apps/web/src/components/TableSizeDialog.tsx">
// "3 × 2", typed rather than clicked.
//
// The toolbar can add a row and remove a row, which is the right shape for
// editing a table and the wrong shape for MAKING one: a person who wants a
// 12 × 5 table is not going to press "+ Row" eleven times, and if that is the
// only way, they will type the CSV by hand instead — which is the outcome the
// grid exists to make unnecessary.
//
// So the size indicator, which already says what the size IS, is also where
// it is set. That is the whole idea: the number that reports a fact is the
// number you edit to change it.
//
// Shrinking is a destructive act and says so before it happens. Not a
// confirm-then-do dialog, which trains people to click through — the count of
// what is about to go is simply on screen next to the button, while the
// number is still being typed.

import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiActor, UiCommand, UiOutcome } from "../lib/uiActions";
import { useId, useState } from "react";

/**
 * The most cells a table may be set to.
 *
 * The same 20,000 the formula route refuses past, for a related reason: a
 * grid that big is a dataset rather than a spreadsheet, and drawing it would
 * lock the window up for a table nobody can read anyway. A person who wants
 * one wants a script to write it.
 */
export const MAX_CELLS = 20_000;

export interface TableSizeDialogProps {
  rows: number;
  columns: number;
  /** How many cells hold something now, per row and per column, so shrinking
   * can say what it is about to take rather than only how much. */
  filledRows: number;
  filledColumns: number;
  onApply: (rows: number, columns: number, actor?: UiActor) => void | boolean;
  sourceCurrent?: () => boolean;
  owner?: string;
  onCancel: () => void;
}

export function TableSizeDialog({
  rows,
  columns,
  filledRows,
  filledColumns,
  onApply,
  sourceCurrent,
  owner,
  onCancel,
}: TableSizeDialogProps) {
  // Held as text, not as numbers: a number input that snaps an empty field
  // back to 1 cannot be cleared and retyped, which is the first thing anybody
  // does to it.
  const [wantRows, setWantRows] = useState(String(rows));
  const [wantColumns, setWantColumns] = useState(String(columns));

  const asked = { rows: Number(wantRows), columns: Number(wantColumns) };
  const whole = (value: number) => Number.isInteger(value) && value >= 1;
  const valid = whole(asked.rows) && whole(asked.columns);
  const tooBig = valid && asked.rows * asked.columns > MAX_CELLS;

  const losingRows = valid ? Math.max(0, filledRows - asked.rows) : 0;
  const losingColumns = valid ? Math.max(0, filledColumns - asked.columns) : 0;

  const parent = `table-size:${useId()}`;
  const [error, setError] = useState("");
  const current = () => sourceCurrent?.() ?? true;
  const revision = useUiRevision(() => [rows, columns, filledRows, filledColumns, wantRows, wantColumns, current(), error]);
  const submit = (actor: UiActor): UiOutcome => {
    if (!valid || tooBig || !current()) return { status: "unavailable", reason: "Choose valid dimensions for the current source (at most 20000 cells)" };
    const result = onApply(asked.rows, asked.columns, actor);
    return result === false ? { status: "unavailable", reason: "Owning table refused the resize; inspect current source" } : { status: "completed", value: { resize_handler_returned: true, source_write: result === true ? "dispatched" : "unconfirmed; inspect source", saved: "not established" } };
  };
  const commands: { key: string; command: UiCommand }[] = [
    { key: `${parent}:draft`, command: { label: "Set table size draft", parent, subject: parent, parameters: { rows: { type: "string", maxLength: 32 }, columns: { type: "string", maxLength: 32 } }, prerequisites: [], read: () => ({ revision: revision(), value: { owner: owner ?? null }, enabled: true }), execute: async args => { setWantRows(args.rows as string); setWantColumns(args.columns as string); return { status: "completed", value: { draft_changed: true, resized: false } }; } } },
    { key: `${parent}:apply`, command: { label: "Apply table size", parent, subject: parent, parameters: {}, prerequisites: [], read: () => ({ revision: revision(), value: { owner: owner ?? null }, enabled: valid && !tooBig && current(), reason: !current() ? "Owning source changed/closed/read-only" : !valid || tooBig ? "Use positive whole dimensions and at most 20000 cells" : undefined }), execute: async (_args, actor) => submit(actor) } },
    { key: `${parent}:cancel`, command: { label: "Cancel table size dialog", parent, subject: parent, parameters: {}, prerequisites: [], read: () => ({ revision: revision(), value: { owner: owner ?? null }, enabled: true }), execute: async () => { onCancel(); return { status: "completed", value: { close_requested: true } }; } } },
    { key: `${parent}:read`, command: { label: "Read table size state", parent, subject: parent, parameters: {}, prerequisites: [], read: () => ({ revision: revision(), value: { owner: owner ?? null }, enabled: true }), execute: async () => ({ status: "completed", value: { owner: owner ?? null, rows, columns, draft_rows: wantRows, draft_columns: wantColumns, valid, too_big: tooBig, losing_rows: losingRows, losing_columns: losingColumns, write_scope_current: current(), error } }) } },
  ];
  const invoke = useUiCommands(commands);
  const click = (key: string) => { void invoke(`${parent}:${key}`).catch(reason => setError(String(reason))); };
  return (
    <div
      className="table-size"
      role="dialog"
      aria-label="Table size"
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          event.preventDefault();
          click("cancel");
        }
      }}
    >
      {error && <p role="alert">{error}</p>}
      <form
        className="table-size__form"
        onSubmit={(event) => {
          event.preventDefault();
          if (!valid || tooBig) return;
          click("apply");
        }}
      >
        <label className="table-size__field">
          Rows
          {/* eslint-disable-next-line jsx-a11y/no-autofocus */}
          <input
            autoFocus
            className="table-size__number"
            type="number"
            min={1}
            inputMode="numeric"
            value={wantRows}
            onChange={(event) => setWantRows(event.target.value)}
          />
        </label>
        <span className="table-size__by" aria-hidden="true">
          ×
        </span>
        <label className="table-size__field">
          Columns
          <input
            className="table-size__number"
            type="number"
            min={1}
            inputMode="numeric"
            value={wantColumns}
            onChange={(event) => setWantColumns(event.target.value)}
          />
        </label>
        <button type="submit" className="btn btn-small" disabled={!valid || tooBig || !current()}>
          Apply
        </button>
        <button type="button" className="btn btn-small" onClick={() => click("cancel")}>
          Cancel
        </button>
      </form>

      {/* One line, and only when it has something to say. A dialog that always
          carries a warning is a dialog nobody reads. */}
      {!valid ? (
        <p className="table-size__note muted" role="status">
          A table is at least one row by one column. Whole numbers only.
        </p>
      ) : tooBig ? (
        <p className="table-size__note table-size__note--bad" role="status">
          {asked.rows.toLocaleString()} × {asked.columns.toLocaleString()} is{" "}
          {(asked.rows * asked.columns).toLocaleString()} cells, past the{" "}
          {MAX_CELLS.toLocaleString()}-cell limit. A grid that size is a dataset rather than a
          spreadsheet — write it from an exec cell and point the table at the file.
        </p>
      ) : losingRows > 0 || losingColumns > 0 ? (
        <p className="table-size__note table-size__note--warn" role="status">
          Removes {describe(losingRows, "row")}
          {losingRows > 0 && losingColumns > 0 ? " and " : ""}
          {describe(losingColumns, "column")}, and what is in{" "}
          {losingRows + losingColumns === 1 ? "it" : "them"}.
        </p>
      ) : null}
    </div>
  );
}

/** "1 row", "3 rows", or nothing at all — so the sentence above reads as a
 * sentence in every case rather than as "Removes 0 rows and 2 columns". */
function describe(count: number, noun: string): string {
  if (count === 0) return "";
  return `${count} ${noun}${count === 1 ? "" : "s"}`;
}
</book:entry>

</book:doc>
