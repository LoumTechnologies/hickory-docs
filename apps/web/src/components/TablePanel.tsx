// A table, edited as a grid.
//
// The point of this component is that the file underneath stays a CSV file
// somebody else will read in a diff. So every edit goes back through
// `writeCsv`, which quotes only what has to be quoted and keeps the line
// ending the file came with — a one-cell edit is a one-line change, and the
// review stays reviewable. See lib/csv.ts.
//
// It is a real grid, not a list of inputs, because the thing that makes a
// table editor worth having is moving around it without the mouse. Tab and
// the arrows move between cells; Enter commits and drops down a row, which is
// what every spreadsheet has trained people to expect. A cell is a plain span
// until you enter it and an input while you are in it — a grid of a thousand
// live inputs is a grid that scrolls badly and reads worse.

import { useCallback, useEffect, useRef, useState } from "react";

import {
  cellAt,
  columnCount,
  insertColumn,
  insertRow,
  parseCsv,
  removeColumn,
  removeRow,
  withCell,
  writeCsv,
  type Csv,
} from "../lib/csv";

export interface TablePanelProps {
  /** The CSV source, exactly as it sits in the document. */
  source: string;
  /** Whether the first row names the columns. */
  header?: boolean;
  /** Forced delimiter; guessed from the text when absent. */
  delimiter?: string;
  /** The new CSV text after an edit. Absent means read-only — a generated
   * table has a document behind it, and editing the output is not the way. */
  onChange?: (csv: string) => void;
}

interface Cursor {
  row: number;
  column: number;
}

export function TablePanel({
  source,
  header = true,
  delimiter,
  onChange,
}: TablePanelProps) {
  // Parsed from the source on every render rather than held as state: the
  // document is the truth, and a second copy here would drift the moment
  // somebody edited the CSV as text instead.
  const table: Csv = parseCsv(source, delimiter);
  const width = Math.max(1, columnCount(table));
  const height = Math.max(1, table.rows.length);
  const editable = onChange !== undefined;

  const [cursor, setCursor] = useState<Cursor | null>(null);
  const [draft, setDraft] = useState("");
  const inputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    if (cursor) inputRef.current?.focus();
  }, [cursor]);

  const apply = useCallback(
    (next: Csv) => {
      onChange?.(writeCsv(next));
    },
    [onChange],
  );

  /** Write the draft into the cell it belongs to, if it changed anything. */
  const commit = useCallback(
    (at: Cursor, value: string) => {
      if (cellAt(table, at.row, at.column) === value) return;
      apply(withCell(table, at.row, at.column, value));
    },
    [apply, table],
  );

  const enter = (row: number, column: number) => {
    if (!editable) return;
    setCursor({ row, column });
    setDraft(cellAt(table, row, column));
  };

  const move = (at: Cursor, dRow: number, dColumn: number) => {
    commit(at, draft);
    const row = Math.max(0, Math.min(at.row + dRow, height - 1));
    const column = Math.max(0, Math.min(at.column + dColumn, width - 1));
    setCursor({ row, column });
    setDraft(cellAt(table, row, column));
  };

  const headerRow = header ? table.rows[0] : null;
  const bodyFrom = header ? 1 : 0;

  return (
    <div className="table-panel" data-testid="table-panel">
      <div className="table-panel__scroll">
        <table className="table-panel__grid">
          {headerRow && (
            <thead>
              <tr>
                {Array.from({ length: width }, (_, column) => (
                  <th key={column} scope="col">
                    <Cell
                      value={cellAt(table, 0, column)}
                      editing={cursor?.row === 0 && cursor.column === column}
                      draft={draft}
                      editable={editable}
                      inputRef={inputRef}
                      onEnter={() => enter(0, column)}
                      onDraft={setDraft}
                      onMove={(dRow, dColumn) => move({ row: 0, column }, dRow, dColumn)}
                      onDone={() => {
                        if (cursor) commit(cursor, draft);
                        setCursor(null);
                      }}
                    />
                  </th>
                ))}
              </tr>
            </thead>
          )}
          <tbody>
            {Array.from({ length: Math.max(0, height - bodyFrom) }, (_, i) => {
              const row = i + bodyFrom;
              return (
                <tr key={row}>
                  {Array.from({ length: width }, (_, column) => (
                    <td key={column}>
                      <Cell
                        value={cellAt(table, row, column)}
                        editing={cursor?.row === row && cursor.column === column}
                        draft={draft}
                        editable={editable}
                        inputRef={inputRef}
                        onEnter={() => enter(row, column)}
                        onDraft={setDraft}
                        onMove={(dRow, dColumn) => move({ row, column }, dRow, dColumn)}
                        onDone={() => {
                          if (cursor) commit(cursor, draft);
                          setCursor(null);
                        }}
                      />
                    </td>
                  ))}
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      {editable ? (
        <div className="table-panel__actions" role="toolbar" aria-label="Table">
          <button
            type="button"
            className="btn btn-small"
            onClick={() => apply(insertRow(table, height))}
            data-tip="Add a row at the bottom"
          >
            + Row
          </button>
          <button
            type="button"
            className="btn btn-small"
            onClick={() => apply(insertColumn(table, width))}
            data-tip="Add a column at the right"
          >
            + Column
          </button>
          <button
            type="button"
            className="btn btn-small"
            disabled={!cursor || height <= 1}
            onClick={() => {
              if (!cursor) return;
              apply(removeRow(table, cursor.row));
              setCursor(null);
            }}
            data-tip="Remove the row the cursor is in"
          >
            − Row
          </button>
          <button
            type="button"
            className="btn btn-small"
            disabled={!cursor || width <= 1}
            onClick={() => {
              if (!cursor) return;
              apply(removeColumn(table, cursor.column));
              setCursor(null);
            }}
            data-tip="Remove the column the cursor is in"
          >
            − Column
          </button>
          <span className="table-panel__size muted">
            {height} × {width}
          </span>
        </div>
      ) : (
        // A generated table's source is a document, and editing the output is
        // not the way to change it. Saying so beats a grid that silently
        // discards what you type.
        <p className="table-panel__readonly muted">
          Written by a document — edit it there.
        </p>
      )}
    </div>
  );
}

function Cell({
  value,
  editing,
  draft,
  editable,
  inputRef,
  onEnter,
  onDraft,
  onMove,
  onDone,
}: {
  value: string;
  editing: boolean;
  draft: string;
  editable: boolean;
  inputRef: React.MutableRefObject<HTMLInputElement | null>;
  onEnter: () => void;
  onDraft: (value: string) => void;
  onMove: (dRow: number, dColumn: number) => void;
  onDone: () => void;
}) {
  if (!editing) {
    return (
      <span
        className="table-panel__cell"
        role={editable ? "button" : undefined}
        tabIndex={editable ? 0 : undefined}
        onClick={editable ? onEnter : undefined}
        onFocus={editable ? onEnter : undefined}
      >
        {/* A non-breaking space, so an empty cell is still a target with a
            height. A zero-height row is a row nobody can click into. */}
        {value === "" ? " " : value}
      </span>
    );
  }
  return (
    <input
      ref={inputRef}
      className="table-panel__input"
      value={draft}
      onChange={(event) => onDraft(event.target.value)}
      onBlur={onDone}
      onKeyDown={(event) => {
        // The chords every spreadsheet has trained people to expect. Enter
        // commits and drops a row; Tab commits and moves right.
        if (event.key === "Enter") {
          event.preventDefault();
          onMove(event.shiftKey ? -1 : 1, 0);
        } else if (event.key === "Tab") {
          event.preventDefault();
          onMove(0, event.shiftKey ? -1 : 1);
        } else if (event.key === "Escape") {
          event.preventDefault();
          onDone();
        } else if (event.key === "ArrowUp" || event.key === "ArrowDown") {
          // The arrows move BETWEEN cells rather than within the text: a
          // grid where Up puts the caret at the start of the field is a grid
          // nobody can navigate.
          event.preventDefault();
          onMove(event.key === "ArrowDown" ? 1 : -1, 0);
        }
      }}
    />
  );
}
