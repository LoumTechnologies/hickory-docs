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

import { api } from "../api/client";
import { cellLabel, isFormula } from "../lib/cellRef";

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
  /** The language this table's formulas are written in. Absent means the
   * table has no formulas: a cell beginning with `=` is then just text, which
   * is what it was before formulas existed and what a table of shell snippets
   * still needs it to be. */
  language?: string;
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
  language,
}: TablePanelProps) {
  // Parsed from the source on every render rather than held as state: the
  // document is the truth, and a second copy here would drift the moment
  // somebody edited the CSV as text instead.
  const table: Csv = parseCsv(source, delimiter);
  const width = Math.max(1, columnCount(table));
  const height = Math.max(1, table.rows.length);
  const editable = onChange !== undefined;

  // What the formulas came to. Held here rather than written into the CSV:
  // the file keeps the FORMULA, which is the thing worth reviewing and the
  // thing that still works when the machine has no interpreter. The value is
  // a view of it, recomputed, exactly as a spreadsheet shows a cell.
  const [computed, setComputed] = useState<{
    values: Record<string, string>;
    errors: Record<string, string>;
  }>({ values: {}, errors: {} });
  const [formulaNote, setFormulaNote] = useState<string | null>(null);

  const hasFormulas = table.rows.some((row) => row.some(isFormula));

  useEffect(() => {
    if (!language || !hasFormulas) {
      setComputed({ values: {}, errors: {} });
      setFormulaNote(null);
      return;
    }
    let live = true;
    // A keystroke is not a question. Evaluating on every character would
    // spawn an interpreter per keypress and show numbers flickering through
    // half-typed expressions.
    const timer = window.setTimeout(() => {
      api.evaluateFormulas(language, table.rows).then(
        (answer) => {
          if (!live) return;
          setComputed(answer);
          setFormulaNote(null);
        },
        (error: unknown) => {
          if (!live) return;
          // A missing interpreter is the common case and not a fault: the
          // table still renders and still edits, it just does not compute.
          setComputed({ values: {}, errors: {} });
          setFormulaNote(error instanceof Error ? error.message : String(error));
        },
      );
    }, 400);
    return () => {
      live = false;
      window.clearTimeout(timer);
    };
    // `source` stands in for the rows, which are rebuilt per render.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [language, source, hasFormulas]);

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

  /**
   * What a cell shows when it is not being edited.
   *
   * A formula shows its VALUE and reveals its source when you enter it —
   * which is what every spreadsheet does, and the only arrangement in which
   * a table of formulas is readable. A formula with no value yet (nothing
   * evaluated, or an error) shows its own text, because a blank cell would
   * be a lie about there being nothing there.
   */
  const shownAt = (row: number, column: number): string => {
    const raw = cellAt(table, row, column);
    if (!isFormula(raw)) return raw;
    return computed.values[cellLabel(column, row)] ?? raw;
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
                      shown={shownAt(0, column)}
                      problem={computed.errors[cellLabel(column, 0)]}
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
                        shown={shownAt(row, column)}
                        problem={computed.errors[cellLabel(column, row)]}
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
          {language && (
            <span className="table-panel__lang muted" data-tip={`Formulas are ${language}`}>
              {language}
            </span>
          )}
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
      {formulaNote && (
        <p className="table-panel__note muted" role="status">
          {formulaNote}
        </p>
      )}
    </div>
  );
}

function Cell({
  value,
  shown,
  problem,
  editing,
  draft,
  editable,
  inputRef,
  onEnter,
  onDraft,
  onMove,
  onDone,
}: {
  /** The cell's own text — a formula, or a literal. */
  value: string;
  /** What to display: a formula's computed value, or its text. */
  shown: string;
  /** What the language said, when this formula did not evaluate. */
  problem?: string;
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
        className={`table-panel__cell${problem ? " table-panel__cell--bad" : ""}${
          value !== shown ? " table-panel__cell--computed" : ""
        }`}
        role={editable ? "button" : undefined}
        tabIndex={editable ? 0 : undefined}
        // A computed cell's hover shows the formula behind it; a broken one
        // shows the LANGUAGE's own message, because `#VALUE!` throws away the
        // only part the author can act on.
        data-tip={problem ?? (value !== shown ? value : undefined)}
        onClick={editable ? onEnter : undefined}
        onFocus={editable ? onEnter : undefined}
      >
        {/* A non-breaking space, so an empty cell is still a target with a
            height. A zero-height row is a row nobody can click into. */}
        {shown === "" ? " " : shown}
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
