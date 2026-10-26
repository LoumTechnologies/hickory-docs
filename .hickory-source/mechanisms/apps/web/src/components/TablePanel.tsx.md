<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/TablePanel.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-38d5395d7736"></a>
## `apps/web/src/components/TablePanel.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A fill handle extends formulas](../../../../../guarantees/authoring/a-fill-handle-extends-formulas/contract.md), [A Table Is A Dataset And A Paragraph](../../../../../guarantees/authoring/a-table-is-a-dataset-and-a-paragraph/contract.md), [Cell formatting keeps numbers numeric](../../../../../guarantees/authoring/cell-formatting-keeps-numbers-numeric/contract.md), [A Formula Is An Expression In A Real Language](../../../../../guarantees/execution/a-formula-is-an-expression-in-a-real-language/contract.md), [Stepping A Table Replays The Order The Host Chose](../../../../../guarantees/execution/stepping-a-table-replays-the-order-the-host-chose/contract.md).

<a id="f-38d5395d7736-TableLayout"></a>
### `TableLayout`

<storycode:copy id="f-38d5395d7736-TableLayout">
import { useTableGestureActions, type TableClipboard } from "./useTableGestureActions";
import { SelectAll, Resizer, HeightResizer } from "./TableGridControls";
import { useCallback, useEffect, useId, useMemo, useRef, useState } from "react";

import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiActor, UiCommand, UiOutcome } from "../lib/uiActions";
import { fillExtent, fillTable } from "../lib/tableFill";
import { currencyInput, displayNumber, readFormats, writeFormats } from "../lib/tableFormats";
import { TableFormatControl } from "./TableFormatControl";
import { TableFillHandle } from "./TableFillHandle";
import { useTableFill } from "./useTableFill";
import { api } from "../api/client";
import type { FormulaStep } from "../api/types";
import {
  cellLabel,
  columnLabel,
  isFormula,
  parseCellLabel,
} from "../lib/cellRef";
import { pointAt, type PointedAt } from "../lib/formulaPoint";
import { moveFormulaReferences } from "../lib/formulaReferences";
import { columnsPhrase, rowsPhrase, tableMenuItems } from "../lib/tableMenu";
import {
  parseClipboardTable,
  toHtmlTable,
  toTabSeparated,
} from "../lib/tableClipboard";
import {
  allSelection,
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
  type Cell as CellAt,
  type Selection,
} from "../lib/tableSelection";

import { fittedRows, intrinsic } from "../lib/tableFit";

import { ContextMenu } from "./ContextMenu";
import { FormulaDebugger } from "./FormulaDebugger";
import { TableSizeDialog } from "./TableSizeDialog";
import { TableCell } from "./TableCell";
import { UiCheckbox } from "./UiCheckbox";
import { FeatureSettings } from "./FeatureSettings";
import { useTableLayout } from "./useTableLayout";

import {
  cellAt,
  clearCells,
  columnCount,
  filledSize,
  insertColumnWithFormulaReferences,
  insertColumn,
  insertRowWithFormulaReferences,
  insertRow,
  parseCsv,
  pasteBlock,
  removeColumns,
  removeRows,
  resizeTable,
  withCell,
  writeCsv,
  type Csv,
} from "../lib/csv";

/**
 * How big a table was left, per table.
 *
 * Presentation, so it is NOT written into the CSV: the file is a dataset a
 * script reads, and a column width is not something it has an opinion about.
 * It lives in the workspace's own state, keyed by the table's `path` when it
 * has one — see lib/uiState.ts.
 */
export interface TableLayout {
  /** Wrap and distribute columns inside the document's prose measure. */
  fitProse?: boolean;
  /** The grid's visible height in pixels. Absent means the default measure. */
  height?: number;
  /** Column widths in pixels, keyed by column index. */
  widths?: Record<string, number>;
  /** Row heights in pixels, keyed by row index. Absent rows are the default
   * measure, which is what nearly every row is. */
  heights?: Record<string, number>;
}

</storycode:copy>
<a id="f-38d5395d7736-TablePanelProps"></a>
### `TablePanelProps`

<storycode:copy id="f-38d5395d7736-TablePanelProps">
export interface TablePanelProps {
  /** The CSV source, exactly as it sits in the document. */
  source: string;
  /** Whether the first row names the columns. */
  header?: boolean;
  /** Forced delimiter; guessed from the text when absent. */
  delimiter?: string;
  /** The new CSV text after an edit. Absent means read-only — a generated
   * table has a document behind it, and editing the output is not the way. */
  onChange?: (csv: string, formats?: string, actor?: UiActor) => void;
  /** Live owning-editor guard; a view refresh alone cannot authorize a stale source write. */
  sourceCurrent?: () => boolean;
  sourceIdentity?: string;
  /** Present only when the document can persist number formats. */
  formats?: string;
  /** The language this table's formulas are written in. Absent means the
   * table has no formulas: a cell beginning with `=` is then just text, which
   * is what it was before formulas existed and what a table of shell snippets
   * still needs it to be. */
  language?: string;
  /** How big this table was left last time. Seeded once — the panel owns it
   * from then on and reports changes through `onLayout`. */
  layout?: TableLayout;
  /** Remember a new size. Absent means the size is this session's only. */
  onLayout?: (next: TableLayout) => void;
  /** Row numbers alongside both the first column and the document rail. */
  laneRight?: boolean;
  /** Markdown tables default to fitting the prose measure; CSV does not. */
  fitProseDefault?: boolean;
}

</storycode:copy>
<a id="f-38d5395d7736-times"></a>
### `times`

Apply `step` to the table `n` times (at least once).
Inserting three rows is inserting one row three times at the same index —
which is also how a spreadsheet reads "select three rows, insert". Kept out
of the component because it is arithmetic, not state.

<storycode:copy id="f-38d5395d7736-times">
/** Apply `step` to the table `n` times (at least once).
 *
 * Inserting three rows is inserting one row three times at the same index —
 * which is also how a spreadsheet reads "select three rows, insert". Kept out
 * of the component because it is arithmetic, not state. */
function times(table: Csv, n: number, step: (t: Csv) => Csv): Csv {
  let out = table;
  for (let i = 0; i < Math.max(1, n); i++) out = step(out);
  return out;
}

</storycode:copy>
<a id="f-38d5395d7736-MIN_COLUMN_WIDTH"></a>
### `MIN_COLUMN_WIDTH`

Narrow enough to tuck a column out of the way, wide enough to grab again.

<storycode:copy id="f-38d5395d7736-MIN_COLUMN_WIDTH">
/** Narrow enough to tuck a column out of the way, wide enough to grab again. */
const MIN_COLUMN_WIDTH = 40;
</storycode:copy>
<a id="f-38d5395d7736-MIN_ROW_HEIGHT"></a>
### `MIN_ROW_HEIGHT`

Short enough to squeeze a row down to a line, tall enough to grab again.

<storycode:copy id="f-38d5395d7736-MIN_ROW_HEIGHT">
/** Short enough to squeeze a row down to a line, tall enough to grab again. */
const MIN_ROW_HEIGHT = 16;
</storycode:copy>
<a id="f-38d5395d7736-MAX_COLUMN_WIDTH"></a>
### `MAX_COLUMN_WIDTH`

The ceilings, matching what `readTableLayout` will store: a fit is driven
by content, and one enormous cell must not make a column nobody can scroll
past.

<storycode:copy id="f-38d5395d7736-MAX_COLUMN_WIDTH">
/** The ceilings, matching what `readTableLayout` will store: a fit is driven
 * by content, and one enormous cell must not make a column nobody can scroll
 * past. */
const MAX_COLUMN_WIDTH = 2000;
</storycode:copy>
<a id="f-38d5395d7736-FIT_BORDER"></a>
### `FIT_BORDER`

The slack on a fit, which is not the same on both axes.
One pixel is the cell's own border, which the measurement does not include
and the declared size does.
A WIDTH gets one more, because rounding a fractional measurement up can
still land on the text and a column set to precisely its content clips the
last letter into an ellipsis — measured in a browser: text whose
`scrollWidth` reports 229 has an intrinsic width of 230. A height has no
ellipsis to fall foul of, and the extra pixel there would be worse than
useless: one line of this font measures 23, so a border and nothing else
puts a fitted row at exactly the 24 it started at, and double-clicking a
row that already fits leaves it alone.

<storycode:copy id="f-38d5395d7736-FIT_BORDER">
/**
 * The slack on a fit, which is not the same on both axes.
 *
 * One pixel is the cell's own border, which the measurement does not include
 * and the declared size does.
 *
 * A WIDTH gets one more, because rounding a fractional measurement up can
 * still land on the text and a column set to precisely its content clips the
 * last letter into an ellipsis — measured in a browser: text whose
 * `scrollWidth` reports 229 has an intrinsic width of 230. A height has no
 * ellipsis to fall foul of, and the extra pixel there would be worse than
 * useless: one line of this font measures 23, so a border and nothing else
 * puts a fitted row at exactly the 24 it started at, and double-clicking a
 * row that already fits leaves it alone.
 */
const FIT_BORDER = 1;
</storycode:copy>
<a id="f-38d5395d7736-FIT_ANTI_CLIP"></a>
### `FIT_ANTI_CLIP`

<storycode:copy id="f-38d5395d7736-FIT_ANTI_CLIP">
const FIT_ANTI_CLIP = 1;
</storycode:copy>
<a id="f-38d5395d7736-ROW_HEIGHT"></a>
### `ROW_HEIGHT`

How far the pointer has to travel before a press on a grid line is a DRAG
rather than a click.
This is what stops the handles from being dead zones. A grid line runs
along the edge of a cell somebody also wants to click, and a five-pixel
strip that swallowed clicks would make the bottom of every row unselectable
— so a press that does not move falls through to the cell underneath.
Below this the grid is the furniture and none of the data.

<storycode:copy id="f-38d5395d7736-ROW_HEIGHT">
/** How far the pointer has to travel before a press on a grid line is a DRAG
 * rather than a click.
 *
 * This is what stops the handles from being dead zones. A grid line runs
 * along the edge of a cell somebody also wants to click, and a five-pixel
 * strip that swallowed clicks would make the bottom of every row unselectable
 * — so a press that does not move falls through to the cell underneath. */
/** Below this the grid is the furniture and none of the data. */
const ROW_HEIGHT = 24;
</storycode:copy>
<a id="f-38d5395d7736-HEAD_HEIGHT"></a>
### `HEAD_HEIGHT`

<storycode:copy id="f-38d5395d7736-HEAD_HEIGHT">
const HEAD_HEIGHT = 22;
</storycode:copy>
<a id="f-38d5395d7736-VISIBLE_ROWS"></a>
### `VISIBLE_ROWS`

<storycode:copy id="f-38d5395d7736-VISIBLE_ROWS">
const VISIBLE_ROWS = 9;
</storycode:copy>
<a id="f-38d5395d7736-MOVE_CLIPBOARD_TYPE"></a>
### `MOVE_CLIPBOARD_TYPE`

<storycode:copy id="f-38d5395d7736-MOVE_CLIPBOARD_TYPE">
const MOVE_CLIPBOARD_TYPE = "application/x-hickory-table-move";

</storycode:copy>
<a id="f-38d5395d7736-clampTo"></a>
### `clampTo`

<storycode:copy id="f-38d5395d7736-clampTo">
const clampTo = (value: number, least: number, most: number) =>
  Math.max(least, Math.min(most, Math.round(value)));

</storycode:copy>
<a id="f-38d5395d7736-TablePanel"></a>
### `TablePanel`

<storycode:copy id="f-38d5395d7736-TablePanel">
export function TablePanel({
  source,
  header = true,
  delimiter,
  onChange,
  sourceCurrent,
  sourceIdentity,
  formats,
  language,
  layout,
  onLayout,
  laneRight = false,
  fitProseDefault = false,
}: TablePanelProps) {
  // Parsed from the source on every render rather than held as state: the
  // document is the truth, and a second copy here would drift the moment
  // somebody edited the CSV as text instead.
  const parent = `table:${useId()}`;
  const [operationError, setOperationError] = useState("");
  const sourceIsCurrent = () => sourceCurrent?.() ?? true;
  const table: Csv = { ...parseCsv(source, delimiter), formats: readFormats(formats) };
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
    source?: string; language?: string; status?: "ok" | "failed";
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
          setComputed({ ...answer, source, language, status: "ok" });
          setFormulaNote(null);
        },
        (error: unknown) => {
          if (!live) return;
          // A missing interpreter is the common case and not a fault: the
          // table still renders and still edits, it just does not compute.
          setComputed({ values: {}, errors: {}, source, language, status: "failed" });
          setFormulaNote(
            error instanceof Error ? error.message : String(error),
          );
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

  // What is selected, which survives losing focus — see the note at the top.
  // A rectangle, whose ANCHOR is the active cell.
  const [selection, setSelection] = useState<Selection | null>(null);
  const active: CellAt | null = selection?.anchor ?? null;
  // Whether the active cell is being typed into. Selected-and-not-editing is
  // the state a spreadsheet spends most of its time in.
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState("");
  // Which surface began the edit, so focus is not yanked out from under the
  // formula bar the moment typing there turns editing on.
  const [entryFrom, setEntryFrom] = useState<"grid" | "bar">("grid");
  // Ctrl+` — show every formula's text instead of its value, which is how you
  // read a sheet somebody else built.
  const [showFormulas, setShowFormulas] = useState(false);
  // The cell whose turn it is, while somebody is stepping through the
  // formulas. Null when the debugger is closed.
  const [stepping, setStepping] = useState<FormulaStep | null>(null);
  const [debugging, setDebugging] = useState(false);
  // Whether the size indicator has been opened to type a new size into.
  const [sizing, setSizing] = useState(false);
  // Where the right-click menu is, in viewport coordinates, or null.
  const [menu, setMenu] = useState<{ x: number; y: number } | null>(null);

  const { gridRef, size, resize, fitProse, setFitProse, columnWidth, rowHeight, widen, heighten } =
    useTableLayout(layout, onLayout, fitProseDefault, source);
  /** How tall the scrolling box is: what was remembered, or the first nine
   * rows once there are more than nine, or nothing at all for a table that
   * fits. Summed rather than multiplied, because a row that was dragged
   * taller has to still be a whole row when it is one of the nine. */
  const gridHeight =
    size.height ??
    (height > VISIBLE_ROWS
      ? Array.from({ length: VISIBLE_ROWS }, (_, row) => rowHeight(row)).reduce(
          (total, each) => total + each,
          HEAD_HEIGHT,
        )
      : undefined);

  const inputRef = useRef<HTMLInputElement | null>(null);
  const barRef = useRef<HTMLInputElement | null>(null);
  const selectedRef = useRef<HTMLElement | null>(null);
  // Which axis a drag is sweeping: cells, whole rows, or whole columns. Null
  // when nothing is being dragged.
  const dragging = useRef<Selection["kind"] | null>(null);
  // Where the last pointed-at reference landed in the draft, so the next
  // click replaces it instead of piling up beside it. See lib/formulaPoint.ts.
  const pointed = useRef<PointedAt | null>(null);
  // The cell the formula is currently POINTING at — by a click, or by the
  // arrow keys walking out from the edited cell the way Excel's point mode
  // does. Drawn with its own dashed ring, deliberately not the selection: the
  // selection is still the cell being edited, and the pointer is the operand
  // being chosen for it. Null when the formula is not pointing at anything.
  const [pointer, setPointer] = useState<CellAt | null>(null);
  // Where to leave the caret after a reference is written in. Applied in an
  // effect because the input has not re-rendered with the new text yet.
  const [caretWanted, setCaretWanted] = useState<number | null>(null);
  const rangeAnchor = useRef<CellAt | null>(null);
  const rangeDragged = useRef(false);
  const cut = useRef<{ from: CellAt; rows: number; columns: number } | null>(
    null,
  );

  const activeInput = () =>
    entryFrom === "bar" ? barRef.current : inputRef.current;

  useEffect(() => {
    if (!selection) return;
    if (editing) {
      if (entryFrom === "grid") inputRef.current?.focus();
    } else {
      selectedRef.current?.focus();
    }
  }, [selection, editing, entryFrom]);

  useEffect(() => {
    if (caretWanted === null) return;
    const input = activeInput();
    input?.focus();
    input?.setSelectionRange(caretWanted, caretWanted);
    setCaretWanted(null);
    // `activeInput` is read, not depended on: it is a getter over two refs.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [caretWanted, draft]);

  // A drag ends wherever the button comes up, including outside the grid —
  // otherwise releasing over the toolbar leaves the grid selecting forever.
  useEffect(() => {
    const up = () => {
      dragging.current = null;
    };
    window.addEventListener("mouseup", up);
    return () => window.removeEventListener("mouseup", up);
  }, []);

  const apply = useCallback(
    (next: Csv, inferFormat = true, actor?: UiActor): boolean => {
      if (!sourceIsCurrent()) { setOperationError("Table source changed or closed. Reopen its current rendering before editing."); return false; }
      if (formats !== undefined) {
        const detected = { ...next.formats };
        const rows = next.rows.map((row, r) => row.map((raw, c) => {
          const money = currencyInput(raw);
          if (!money) return raw;
          if (inferFormat) detected[cellLabel(c, r)] = money.format;
          return money.value;
        }));
        next = { ...next, formats: detected, rows };
      }
      const text = writeCsv(next);
      const nextFormats = writeFormats(next.formats);
      // A write that changes nothing still marks the document dirty and still
      // lands in the undo history, which is how pressing Delete on already
      // empty cells ends up as a commit somebody has to explain.
      if (text !== source || (formats !== undefined && nextFormats !== formats)) {
        if (actor?.kind === "agent") onChange?.(text, formats === undefined ? undefined : nextFormats, actor);
        else if (formats === undefined) onChange?.(text);
        else onChange?.(text, nextFormats);
      }
      return true;
    },
    [onChange, source, formats, sourceCurrent],
  );

  const fill = useTableFill({ table, selection, revision: `${source}\0${formats ?? ""}`, language, gridRef,
    onFill: (next, range) => { apply(next, false); setSelection(range); } });

  // `=` is ordinary text until the table declares a formula language.
  const insertTableRow = (current: Csv, at: number) =>
    language
      ? insertRowWithFormulaReferences(current, at)
      : insertRow(current, at);
  const insertTableColumn = (current: Csv, at: number) =>
    language
      ? insertColumnWithFormulaReferences(current, at)
      : insertColumn(current, at);

  /** Write the draft into the cell it belongs to, if it changed anything. */
  const commit = useCallback(
    (at: CellAt, value: string, actor?: UiActor) => {
      if (cellAt(table, at.row, at.column) === value) return true;
      return apply(withCell(table, at.row, at.column, value), true, actor);
    },
    [apply, table],
  );

  /** Commit whatever is being typed and leave edit mode, keeping the cell
   * selected so the toolbar still has something to act on.
   *
   * Every act that moves the selection goes through this rather than merely
   * turning editing off. Dropping the edit instead would lose it silently:
   * the input unmounts the moment editing ends, and React sends no blur to a
   * field that is no longer there — so the blur this used to rely on never
   * arrives when a drag begins somewhere else. */
  const stopEditing = () => {
    if (editing && active) commit(active, draft);
    setEditing(false);
    setPointer(null);
  };

  /** Make a cell active without typing into it — a click, an arrow, a header. */
  const select = (row: number, column: number) => {
    stopEditing();
    setSelection(cellSelection(row, column));
  };

  /** Reach the current selection out to another cell: a drag, or a shift-click.
   * With nothing selected yet there is no anchor to reach from, so this is a
   * plain selection instead. */
  const extend = (row: number, column: number) => {
    stopEditing();
    setSelection((current) =>
      current
        ? extendTo(current, row, column, height, width)
        : cellSelection(row, column),
    );
  };

  /** Start typing into a cell. `seed` replaces its contents, as typing over a
   * selected cell does in every spreadsheet; absent, the cell's own text is
   * the starting point, which is what F2 and a double-click mean. */
  const edit = (
    row: number,
    column: number,
    seed?: string,
    from: "grid" | "bar" = "grid",
  ) => {
    if (!editable) return;
    setSelection(cellSelection(row, column));
    setDraft(seed ?? cellAt(table, row, column));
    setEntryFrom(from);
    setEditing(true);
    // A new edit has pointed at nothing yet; the last edit's reference is not
    // this one's to replace.
    pointed.current = null;
    setPointer(null);
  };

  const moveTo = (row: number, column: number) => {
    const next = {
      row: Math.max(0, Math.min(row, height - 1)),
      column: Math.max(0, Math.min(column, width - 1)),
    };
    setSelection(cellSelection(next.row, next.column));
    return next;
  };

  /** Commit and step, which is what Enter and Tab do while editing. The cell
   * arrived at is selected rather than open: Excel commits, Sheets commits,
   * and an editor that opened the next cell would swallow the next keystroke
   * as a replacement of whatever was already there. */
  const commitAndMove = (at: CellAt, dRow: number, dColumn: number) => {
    commit(at, draft);
    moveTo(at.row + dRow, at.column + dColumn);
    setEditing(false);
    setPointer(null);
  };

  /**
   * What was selected before a press on a grid line.
   *
   * A double-click is two clicks, and the first of them cannot know the second
   * is coming — so the tap it fires has already moved the selection by the
   * time "fit" is the answer. Rather than delay every tap behind a
   * double-click timer, which would make an ordinary press on a line feel
   * broken, the press remembers and the fit puts it back.
   */
  const beforePress = useRef<Selection | null | undefined>(undefined);

  /** A press on a line that turned out to be a click: select what it is
   * beside, remembering what was selected in case a fit follows. */
  const tapped = (take: () => void) => () => {
    beforePress.current = selection;
    take();
  };

  /**
   * A double-click: fit, and give back the selection the press moved.
   * Resizing is not a way of choosing something.
   *
   * `undefined` means no press of this double-click moved anything — a
   * read-only table, where the cell handles have no tap at all — and then the
   * selection is left exactly as it is rather than cleared. Restoring
   * something nobody took would be the same bug the other way round.
   */
  const fitted = (fit: () => void) => () => {
    fit();
    if (beforePress.current !== undefined) setSelection(beforePress.current);
    beforePress.current = undefined;
  };

  // Double-click measures rendered content, then stores a manual width.
  const fitColumn = (column: number) => {
    const cells = gridRef.current?.querySelectorAll<HTMLElement>(
      `[data-column="${column}"]`,
    );
    if (!cells || cells.length === 0) return;
    widen(
      column,
      clampTo(
        intrinsic(cells, "width") + FIT_BORDER + FIT_ANTI_CLIP,
        MIN_COLUMN_WIDTH,
        MAX_COLUMN_WIDTH,
      ),
    );
  };

  /** Row headers fit the selection saved before the first click. */
  const fitRow = (row: number, selected = false) =>
    resize({
      ...size,
      heights: {
        ...size.heights,
        ...fittedRows(
          gridRef.current, row,
          selected ? (beforePress.current === undefined ? selection : beforePress.current) : null,
        ),
      },
    });

  /** Take the whole table — the corner box, and Ctrl+A. */
  const selectAll = () => {
    stopEditing();
    setSelection(allSelection(height, width));
  };

  /** Take a whole column, from its letter. Shift reaches from whatever is
   * already selected, which is how three columns are taken without a drag. */
  const takeColumn = (column: number, shift: boolean) => {
    if (shift && selection) return extend(height - 1, column);
    stopEditing();
    setSelection(columnSelection(column, height));
  };

  /** The same for a whole row, from its number. */
  const takeRow = (row: number, shift: boolean) => {
    if (shift && selection) return extend(row, width - 1);
    stopEditing();
    setSelection(rowSelection(row, width));
  };

  /**
   * Whether a click on another cell would mean "put its reference here".
   *
   * Only while a FORMULA is open, and only in a table that names a language:
   * where `=` is just text there are no references for a click to write.
   */
  const pointing =
    editable && editing && language !== undefined && isFormula(draft);

  /**
   * A click on a cell while a formula is open.
   *
   * Answers whether the click was taken as a reference. When it was not — the
   * expression is not asking for an operand — the caller treats it as the
   * ordinary click it was, which is how you still leave a formula cell by
   * clicking somewhere else.
   */
  const pointTo = (row: number, column: number): boolean => {
    if (!pointing) return false;
    if (active && active.row === row && active.column === column) return false;
    const input = activeInput();
    const caret = input?.selectionStart ?? draft.length;
    const written = pointAt(
      draft,
      caret,
      cellLabel(column, row),
      pointed.current,
    );
    if (!written) return false;
    setDraft(written.text);
    pointed.current = written.pointed;
    setCaretWanted(written.caret);
    setPointer({ row, column });
    return true;
  };

  /** Write the rectangle dragged from `from` to this cell as `B2:C4`. */
  const pointRangeTo = (from: CellAt, row: number, column: number): boolean => {
    if (from.row === row && from.column === column) return pointTo(row, column);
    if (!pointing) return false;
    const input = activeInput();
    const caret = input?.selectionStart ?? draft.length;
    const left = Math.min(from.column, column);
    const top = Math.min(from.row, row);
    const right = Math.max(from.column, column);
    const bottom = Math.max(from.row, row);
    const written = pointAt(
      draft,
      caret,
      `${cellLabel(left, top)}:${cellLabel(right, bottom)}`,
      pointed.current,
    );
    if (!written) return false;
    setDraft(written.text);
    pointed.current = written.pointed;
    setCaretWanted(written.caret);
    setPointer({ row, column });
    return true;
  };

  /** Track a formula-range drag from the table cell, not only its inner text
   * span. The resize edges sit above that span, so `mouseenter` alone misses
   * a real drag that crosses an edge. */
  const pointRangeOver = (row: number, column: number) => {
    if (!pointing || !rangeAnchor.current) return;
    if (
      rangeAnchor.current.row === row &&
      rangeAnchor.current.column === column
    )
      return;
    rangeDragged.current = true;
    pointRangeTo(rangeAnchor.current, row, column);
  };

  // Each navigation step replaces the last reference; edges swallow the step.
  const pointBy = (dRow: number, dColumn: number, caret: number): boolean => {
    if (!pointing || !active) return false;
    const from = pointer ?? active;
    const target = {
      row: Math.max(0, Math.min(from.row + dRow, height - 1)),
      column: Math.max(0, Math.min(from.column + dColumn, width - 1)),
    };
    if (
      pointer &&
      target.row === pointer.row &&
      target.column === pointer.column
    )
      return true;
    const written = pointAt(
      draft,
      caret,
      cellLabel(target.column, target.row),
      pointed.current,
    );
    if (!written) return false;
    setDraft(written.text);
    pointed.current = written.pointed;
    setCaretWanted(written.caret);
    setPointer(target);
    return true;
  };

  // Up/Down point; Left/Right point at edges and otherwise move the caret.
  const pointKey = (event: React.KeyboardEvent<HTMLInputElement>): boolean => {
    if (!pointing) return false;
    const field = event.currentTarget;
    const start = field.selectionStart ?? draft.length;
    const end = field.selectionEnd ?? start;
    const collapsed = start === end;
    // The caret cannot usefully sit before the `=`: a Left at the very start
    // points, inserting where the first operand goes.
    const caret = Math.max(start, Math.min(1, draft.length));
    const live =
      pointer !== null &&
      pointed.current !== null &&
      collapsed &&
      start === pointed.current.end;
    switch (event.key) {
      case "ArrowUp":
        return pointBy(-1, 0, caret);
      case "ArrowDown":
        return pointBy(1, 0, caret);
      case "ArrowLeft":
        if (!live && !(collapsed && start <= 1)) return false;
        return pointBy(0, -1, caret);
      case "ArrowRight":
        if (!live && !(collapsed && start >= draft.length)) return false;
        return pointBy(0, 1, caret);
      default:
        return false;
    }
  };

  /** What typing into the open cell does: the text changes, and any pointer
   * is dropped — the next arrow leaves from the edited cell again, and the
   * next click writes a fresh reference rather than replacing the last. */
  const typed = (text: string) => {
    setPointer(null);
    setDraft(text);
  };

  /**
   * The selected rectangle, as cells.
   *
   * What goes on the clipboard, and what "− Row" would take: the same
   * rectangle the grid is drawing, read out of the table rather than out of
   * anything the clipboard has to be told about separately.
   */
  const blockOf = (of: Selection): string[][] =>
    rowsIn(of).map((row) =>
      columnsIn(of).map((column) => cellAt(table, row, column)),
    );

  /**
   * Ctrl+C, and Ctrl+X, over a selection that is not being typed into.
   *
   * Both flavours go on, because the other end of this is almost never this
   * app: tab-separated text is what a spreadsheet reads, and a real `<table>`
   * is what Excel, Word and Sheets read when they want structure. See
   * lib/tableClipboard.ts.
   *
   * Answers whether it wrote anything, so the caller only takes the event
   * away from the browser when there was something to take it for.
   */
  const copyOut = (clipboard: TableClipboard | null | undefined): boolean => {
    if (!selection || !clipboard) return false;
    const block = blockOf(selection);
    clipboard.setData("text/plain", toTabSeparated(block));
    clipboard.setData("text/html", toHtmlTable(block));
    return true;
  };

  /**
   * Ctrl+V over a selection.
   *
   * Anchored at the anchor and NOT stretched to fill the selection: a
   * spreadsheet repeats a small block to fill a bigger selection, which is a
   * clever rule that silently writes cells nobody looked at. What lands is
   * what was on the clipboard, and the selection afterwards is exactly the
   * cells it wrote — so the first thing you see is what changed, and one
   * Delete puts it back.
   */
  const pasteIn = (clipboard: TableClipboard | null | undefined, actor?: UiActor): boolean => {
    if (!editable || !selection || !clipboard) return false;
    const block = parseClipboardTable(
      clipboard.getData("text/html"),
      clipboard.getData("text/plain"),
    );
    if (!block || block.length === 0) return false;
    const at = selection.anchor;
    let next = pasteBlock(table, at.row, at.column, block);
    if (cut.current && clipboard.getData(MOVE_CLIPBOARD_TYPE) === "1") {
      const moved = cut.current;
      next = {
        ...next,
        rows: next.rows.map((fields) =>
          fields.map((field) =>
            moveFormulaReferences(
              field,
              moved.from,
              { rows: moved.rows, columns: moved.columns },
              at,
            ),
          ),
        ),
      };
      cut.current = null;
    }
    if (!apply(next, false, actor)) return false;
    const tall = block.length;
    const wide = block.reduce((widest, row) => Math.max(widest, row.length), 0);
    setSelection(
      extendTo(
        cellSelection(at.row, at.column),
        at.row + tall - 1,
        at.column + wide - 1,
        // The table this reaches over is the one the paste just made, not the
        // one it landed in — clamping to the old size would leave the
        // selection short of what was written.
        Math.max(height, at.row + tall),
        Math.max(width, at.column + wide),
      ),
    );
    return true;
  };

  // Formatting is a view; editing and formula evaluation use raw values.
  const shownAt = (row: number, column: number): string => {
    const raw = cellAt(table, row, column);
    const label = cellLabel(column, row);
    if (isFormula(raw) && showFormulas) return raw;
    return displayNumber(isFormula(raw) ? computed.values[label] ?? raw : raw, table.formats?.[label]);
  };

  /** Keys handled by a selected-but-not-editing cell: the spreadsheet ones. */
  const onSelectedKeyDown = (event: React.KeyboardEvent, at: CellAt) => {
    const key = event.key;
    if (
      key === "ArrowUp" ||
      key === "ArrowDown" ||
      key === "ArrowLeft" ||
      key === "ArrowRight"
    ) {
      event.preventDefault();
      // Shift REACHES rather than moves, which is how a range is made without
      // a mouse — and the only way to make one on a keyboard at all.
      const from = event.shiftKey && selection ? selection.focus : at;
      const row =
        from.row + (key === "ArrowDown" ? 1 : key === "ArrowUp" ? -1 : 0);
      const column =
        from.column + (key === "ArrowRight" ? 1 : key === "ArrowLeft" ? -1 : 0);
      if (event.shiftKey) extend(row, column);
      else moveTo(row, column);
    } else if (key === "a" && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      selectAll();
    } else if (key === "Tab") {
      event.preventDefault();
      moveTo(at.row, at.column + (event.shiftKey ? -1 : 1));
    } else if (key === "Enter" || key === "F2") {
      // Enter opens the cell, as it does in Sheets. F2 is Excel's chord for
      // the same thing and costs one line to honour.
      event.preventDefault();
      edit(at.row, at.column);
    } else if (key === "Delete" || key === "Backspace") {
      event.preventDefault();
      // Everything selected, not just the anchor: a range you can make is a
      // range you expect Delete to empty.
      if (editable && selection) apply(clearCells(table, cellsIn(selection)));
    } else if (
      // A printable character replaces the cell, which is the fastest thing a
      // spreadsheet does and the one people miss most when it is absent.
      key.length === 1 &&
      !event.ctrlKey &&
      !event.metaKey &&
      !event.altKey
    ) {
      event.preventDefault();
      edit(at.row, at.column, key);
    }
  };

  const selectedRaw = active ? cellAt(table, active.row, active.column) : "";
  const barValue = editing ? draft : selectedRaw;
  const selectedProblem = active
    ? computed.errors[cellLabel(active.column, active.row)]
    : undefined;

  // Which rows and columns a "− Row" or "− Column" would take. Removing every
  // one of them is refused rather than offered: a table with no rows left is
  // not something anybody meant to ask for, and it is what a column selection
  // would do to one.
  const selectedRows = selection ? rowsIn(selection) : [];
  const selectedColumns = selection ? columnsIn(selection) : [];
  const canRemoveRows = selectedRows.length > 0 && selectedRows.length < height;
  const canRemoveColumns =
    selectedColumns.length > 0 && selectedColumns.length < width;

  /**
   * Open the right-click menu over a cell.
   *
   * A right-click on a cell OUTSIDE the current selection selects it first,
   * which is what every spreadsheet does and what stops "Delete row" from
   * quietly meaning some other row that was selected a minute ago. A
   * right-click INSIDE the selection leaves the sweep alone, because taking a
   * three-row selection apart to act on three rows is the opposite of what
   * was asked for.
   */
  const openMenu = (
    event: React.MouseEvent,
    row: number,
    column: number,
    take?: "row" | "column",
  ) => {
    if (!editable) return;
    event.preventDefault();
    if (!selection || !containsCell(selection, row, column)) {
      stopEditing();
      setSelection(
        take === "row"
          ? rowSelection(row, width)
          : take === "column"
            ? columnSelection(column, height)
            : cellSelection(row, column),
      );
    }
    setMenu({ x: event.clientX, y: event.clientY });
  };

  /** Run what the menu was clicked for, against the current selection. */
  const runMenu = (action: string, actor?: UiActor) => {
    setMenu(null);
    if (!selection) return;
    const rows = rowsIn(selection);
    const columns = columnsIn(selection);
    stopEditing();
    switch (action) {
      case "insert-rows-above":
        apply(times(table, rows.length, (t) => insertTableRow(t, rows[0])), true, actor);
        break;
      case "insert-rows-below":
        apply(
          times(table, rows.length, (t) =>
            insertTableRow(t, rows[rows.length - 1] + 1),
          ), true, actor,
        );
        break;
      case "insert-columns-left":
        apply(
          times(table, columns.length, (t) => insertTableColumn(t, columns[0])), true, actor,
        );
        break;
      case "insert-columns-right":
        apply(
          times(table, columns.length, (t) =>
            insertTableColumn(t, columns[columns.length - 1] + 1),
          ), true, actor,
        );
        break;
      case "delete-rows":
        apply(removeRows(table, rows), true, actor);
        setSelection(null);
        break;
      case "delete-columns":
        apply(removeColumns(table, columns), true, actor);
        setSelection(null);
        break;
    }
  };

  // While stepping: the cell whose turn it is, and the cells it read. Both by
  // label, which is what the host answers in.
  const steppingAt = useMemo(
    () => (stepping ? parseCellLabel(stepping.cell) : null),
    [stepping],
  );
  const steppingReads = useMemo(
    () => new Set(stepping?.bindings.map((binding) => binding.cell) ?? []),
    [stepping],
  );
  const onStep = useCallback(
    (step: FormulaStep | null) => setStepping(step),
    [],
  );

  const computation = !language || !hasFormulas ? "not_applicable" : computed.source === source && computed.language === language ? computed.status ?? "pending" : "pending";
  const revision = useUiRevision(() => [sourceIdentity, source, formats, language, header, selection, editing, draft, computed, showFormulas, debugging, stepping, sizing, menu, size, fitProse, sourceIsCurrent(), operationError]);
  const commands: { key: string; command: UiCommand }[] = [];
  const index = { type: "number" as const, minimum: 0, maximum: 1000000, integer: true };
  const add = (key: string, label: string, parameters: UiCommand["parameters"], enabled: boolean, execute: UiCommand["execute"], mutates = false) => commands.push({ key: `${parent}:${key}`, command: { label, parent, subject: parent, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled: enabled && (!mutates || sourceIsCurrent()), reason: !sourceIsCurrent() ? "Owning source changed, closed or became read-only" : !enabled ? "Select a valid target and commit/cancel any current edit first" : undefined }), execute } });
  const written = (ok: boolean, actor: UiActor): UiOutcome => ok ? { status: "completed", value: { source_write: "handler dispatched; inspect actual source", actor, saved: "not established", verification: "not requested" } } : { status: "unavailable", reason: "The owning table refused a stale source write" };
  add("read", "Read table controller state bytes", { offset: { ...index, maximum: 1000000000 }, limit: { ...index, minimum: 1, maximum: 8192 } }, true, async args => {
    const bytes = new TextEncoder().encode(JSON.stringify({ source_identity: sourceIdentity ?? null, rows: height, columns: width, header, editable, selection, editing, draft: editing ? draft : null, show_formulas: showFormulas, formula_note: formulaNote, computation, debugging, stepping, sizing, menu_open: !!menu, layout: size, fit_prose: fitProse, write_scope_current: sourceIsCurrent(), error: operationError, scope: "Local table controller state; formula values and dispatched writes are not independent verification" }));
    const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
    if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside table state" };
    return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
  });
  add("cells", "Read table cell region bytes", { row: index, column: index, rows: { ...index, minimum: 1, maximum: 64 }, columns: { ...index, minimum: 1, maximum: 64 }, offset: { ...index, maximum: 1000000000 }, limit: { ...index, minimum: 1, maximum: 8192 } }, true, async args => {
    const row = args.row as number, column = args.column as number;
    if (row >= height || column >= width) return { status: "unavailable", reason: "Region begins outside the observed table" };
    const cells = [];
    for (let r = row; r < Math.min(height, row + (args.rows as number)); r++) for (let c = column; c < Math.min(width, column + (args.columns as number)); c++) cells.push({ row: r, column: c, label: cellLabel(c, r), raw: cellAt(table, r, c), displayed: shownAt(r, c), error: computed.errors[cellLabel(c, r)] ?? null });
    const bytes = new TextEncoder().encode(JSON.stringify({ cells, total_rows: height, total_columns: width, write_scope_current: sourceIsCurrent(), computation, scope: "Zero-based coordinates; explicit current region only; not a document edit baseline" }));
    const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
    if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside table region" };
    return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
  });
  add("select", "Select table cell or range", { row: index, column: index, extend: { type: "boolean" } }, !editing, async args => {
    if ((args.row as number) >= height || (args.column as number) >= width) return { status: "unavailable", reason: "Cell outside current table" };
    if (args.extend) extend(args.row as number, args.column as number); else select(args.row as number, args.column as number);
    return { status: "completed", value: { selection_changed: true } };
  });
  add("select-row", "Select table row", { row: index, extend: { type: "boolean" } }, !editing, async args => { if ((args.row as number) >= height) return { status: "unavailable", reason: "Row outside table" }; takeRow(args.row as number, args.extend as boolean); return { status: "completed", value: { selection_changed: true } }; });
  add("select-column", "Select table column", { column: index, extend: { type: "boolean" } }, !editing, async args => { if ((args.column as number) >= width) return { status: "unavailable", reason: "Column outside table" }; takeColumn(args.column as number, args.extend as boolean); return { status: "completed", value: { selection_changed: true } }; });
  add("select-all", "Select all table cells", {}, !editing, async () => { selectAll(); return { status: "completed", value: { selection_changed: true } }; });
  add("write", "Write table cell contents", { row: index, column: index, value: { type: "string", maxLength: 1048576 } }, editable && !editing, async (args, actor) => {
    const row = args.row as number, column = args.column as number;
    if (row >= height || column >= width) return { status: "unavailable", reason: "Cell outside table; resize/add first" };
    return written(commit({ row, column }, args.value as string, actor), actor);
  }, true);
  add("commit", "Commit pending table cell edit", {}, editable && editing && !!active, async (_args, actor) => { if (!active) return { status: "unavailable", reason: "No active cell" }; const ok = commit(active, draft, actor); if (ok) { setEditing(false); setPointer(null); } return written(ok, actor); }, true);
  add("cancel-edit", "Cancel pending table cell edit", {}, editing, async () => { setDraft(selectedRaw); setEditing(false); setPointer(null); return { status: "completed", value: { draft_cancelled: true, source_written: false } }; });
  if (editable) {
    add("add-row", "Add table row", {}, !editing, async (_args, actor) => written(apply(insertTableRow(table, height), true, actor), actor), true);
    add("add-column", "Add table column", {}, !editing, async (_args, actor) => written(apply(insertTableColumn(table, width), true, actor), actor), true);
    add("menu", "Set table context menu visibility", { open: { type: "boolean" } }, !!selection && !editing, async args => { setMenu(args.open ? { x: 16, y: 16 } : null); return { status: "completed", value: { open: args.open } }; });
    if (menu && selection) for (const item of tableMenuItems({ rows: selectedRows, columns: selectedColumns, height, width })) add(`menu:${item.id}`, item.label, {}, !item.disabled && !editing, async (_args, actor) => { runMenu(item.id, actor); return { status: "completed", value: { source_write: "handler dispatched; inspect actual source", actor } }; }, true);
    add("fill", "Fill table selection to cell", { row: index, column: index }, !!selection && !editing, async (args, actor) => {
      if (!selection) return { status: "unavailable", reason: "Select a seed region first" };
      const target = fillExtent(table, selection, { row: args.row as number, column: args.column as number });
      const ok = apply(fillTable(table, selection, target, language || false), false, actor); if (ok) setSelection(target);
      return written(ok, actor);
    }, true);
    add("size", "Set table size dialog visibility", { open: { type: "boolean" } }, !editing, async args => { setSizing(args.open as boolean); return { status: "completed", value: args.open }; });
  }
  if (editable && language) {
    add("formulas", "Set table formula visibility", { show: { type: "boolean" } }, true, async args => { setShowFormulas(args.show as boolean); return { status: "completed", value: args.show }; });
    if (hasFormulas) add("debug", "Set formula debugger visibility", { open: { type: "boolean" } }, true, async args => { setDebugging(args.open as boolean); return { status: "completed", value: args.open }; });
  }

  add("column-width", "Resize table column", { column: index, pixels: { type: "number", minimum: MIN_COLUMN_WIDTH, maximum: MAX_COLUMN_WIDTH } }, true, async args => { if ((args.column as number) >= width) return { status: "unavailable", reason: "Column outside table" }; widen(args.column as number, args.pixels as number); return { status: "completed", value: { layout_change_requested: true } }; });
  add("row-height", "Resize table row", { row: index, pixels: { type: "number", minimum: MIN_ROW_HEIGHT, maximum: 2000 } }, true, async args => { if ((args.row as number) >= height) return { status: "unavailable", reason: "Row outside table" }; heighten(args.row as number, args.pixels as number); return { status: "completed", value: { layout_change_requested: true } }; });
  add("fit-column", "Fit table column to rendered contents", { column: index }, true, async args => { if ((args.column as number) >= width) return { status: "unavailable", reason: "Column outside table" }; fitColumn(args.column as number); return { status: "completed", value: { layout_measurement_requested: true } }; });
  add("fit-row", "Fit table row to rendered contents", { row: index }, true, async args => { if ((args.row as number) >= height) return { status: "unavailable", reason: "Row outside table" }; fitRow(args.row as number); return { status: "completed", value: { layout_measurement_requested: true } }; });
  useUiCommands(commands);
  useTableGestureActions({ parent, revision, current: sourceIsCurrent, editable, selected: !!selection, editing, pointing: !!pointing, rows: height, columns: width,
    begin: (row, column, from) => edit(row, column, undefined, from), change: typed, input: activeInput, viewport: () => gridRef.current?.closest<HTMLElement>(".table-panel__scroll") ?? null, copy: copyOut, paste: pasteIn,
    point: (row, column, endRow, endColumn) => pointRangeTo({ row, column }, endRow, endColumn), resize: next => resize({ ...size, height: next }) });
  const rowNumber = (row: number, lane = false) => (
    <th
      scope="row"
      className={`table-panel__gutter${lane ? " table-panel__gutter--lane" : ""}${
        selection && containsCell(selection, row, selection.anchor.column)
          ? " table-panel__gutter--active"
          : ""
      }${
        selection?.kind === "rows" && rowsIn(selection).includes(row)
          ? " table-panel__gutter--taken"
          : ""
      }`}
      data-tip={`Row ${row + 1} — click to select it, drag for more`}
      onMouseDown={(event) => {
        // Selecting on the mousedown is what makes a drag possible at all —
        // by the time a click arrives the sweep is over. The click below is
        // the same act for a pointer that sends no mousedown.
        if (event.button === 2) return;
        event.preventDefault();
        dragging.current = "rows";
        takeRow(row, event.shiftKey);
      }}
      onMouseEnter={() => {
        if (dragging.current === "rows") extend(row, width - 1);
      }}
      onClick={(event) => takeRow(row, event.shiftKey)}
      onContextMenu={(event) => openMenu(event, row, 0, "row")}
    >
      {row + 1}
      <Resizer
        axis="row"
        size={() => rowHeight(row)}
        least={MIN_ROW_HEIGHT}
        onResize={(next) => heighten(row, next)}
        onTap={tapped(() => takeRow(row, false))}
        onFit={fitted(() => fitRow(row, true))}
      />
    </th>
  );

  return (
    <div
      className={`table-panel${fitProse ? " table-panel--fit-prose" : ""}`}
      data-testid="table-panel"
      style={
        {
          "--table-row-height": `${ROW_HEIGHT}px`,
          "--table-head-height": `${HEAD_HEIGHT}px`,
        } as React.CSSProperties
      }
      onKeyDown={(event) => {
        // Excel's chord for "show the formulas, not the answers".
        if (event.key === "`" && (event.ctrlKey || event.metaKey)) {
          event.preventDefault();
          setShowFormulas((on) => !on);
        }
      }}
      onCopy={(event) => {
        if (inAField(event)) return;
        if (copyOut(event.clipboardData)) event.preventDefault();
      }}
      onCut={(event) => {
        if (inAField(event) || !editable || !selection) return;
        if (!copyOut(event.clipboardData)) return;
        event.preventDefault();
        const rows = rowsIn(selection);
        const columns = columnsIn(selection);
        event.clipboardData?.setData(MOVE_CLIPBOARD_TYPE, "1");
        cut.current = {
          from: { row: rows[0], column: columns[0] },
          rows: rows.length,
          columns: columns.length,
        };
        apply(clearCells(table, cellsIn(selection)));
      }}
      onPaste={(event) => {
        if (inAField(event)) return;
        if (pasteIn(event.clipboardData)) event.preventDefault();
      }}
    >
      {/* The name box and the formula bar. The name box is where A1 stops
          being a convention somebody has to be told about, and the bar is
          where a cell's TEXT is always visible even when the grid is showing
          what it came to. Its placeholder is also the only place that says a
          formula begins with `=`. */}
      <div className="table-panel__bar">
        <span
          className="table-panel__name mono"
          data-testid="table-panel-name"
          data-tip={
            selection
              ? isSingleCell(selection)
                ? "The selected cell, in A1 notation"
                : "What is selected, in A1 notation"
              : "No cell selected"
          }
        >
          {selection ? selectionLabel(selection) : "—"}
        </span>
        <span className="table-panel__fx" aria-hidden="true">
          fx
        </span>
        <input
          ref={barRef}
          className="table-panel__formula mono"
          aria-label="Cell contents"
          value={barValue}
          disabled={!editable || !active}
          placeholder={
            !active
              ? "Select a cell"
              : language
                ? `Text, or = for a ${language} expression`
                : "Text — this table names no formula language"
          }
          onChange={(event) => {
            if (!active) return;
            if (editing && entryFrom === "bar") typed(event.target.value);
            else edit(active.row, active.column, event.target.value, "bar");
          }}
          onBlur={stopEditing}
          onKeyDown={(event) => {
            if (!active) return;
            if (event.key.startsWith("Arrow") && pointKey(event)) {
              event.preventDefault();
              return;
            }
            if (event.key === "Enter") {
              event.preventDefault();
              commitAndMove(active, event.shiftKey ? -1 : 1, 0);
            } else if (
              event.key === "ArrowLeft" &&
              (event.currentTarget.selectionStart ?? 0) === 0 &&
              (event.currentTarget.selectionEnd ?? 0) === 0
            ) {
              event.preventDefault();
              commitAndMove(active, 0, -1);
            } else if (
              event.key === "ArrowRight" &&
              (event.currentTarget.selectionStart ?? draft.length) ===
                draft.length &&
              (event.currentTarget.selectionEnd ?? draft.length) ===
                draft.length
            ) {
              event.preventDefault();
              commitAndMove(active, 0, 1);
            } else if (event.key === "Escape") {
              event.preventDefault();
              setDraft(selectedRaw);
              setEditing(false);
            }
          }}
        />
        <FeatureSettings label="Table settings" owner={parent}>
          <label><UiCheckbox label="table prose wrapping" owner={parent} checked={fitProse}
            onCheckedChange={setFitProse} />Wrap to prose width</label>
        </FeatureSettings>
      </div>

      <div
        className="table-panel__scroll"
        data-testid="table-panel-scroll"
        // A remembered height wins over everything: it is what this person
        // decided this table is worth. Failing that, a table past nine rows
        // is pinned to nine and scrolls; a shorter one is left to be its own
        // height, because capping a four-row table would be furniture around
        // nothing.
        style={
          gridHeight === undefined
            ? undefined
            : { height: `${gridHeight}px`, maxHeight: "none" }
        }
      >
        <table
          ref={gridRef}
          style={laneRight || fitProse ? undefined : {
            width: `calc(${Array.from({ length: width }, (_, column) => columnWidth(column)).reduce((a, b) => a + b, 0)}px + 2.6rem)`,
          }}
          className={`table-panel__grid${laneRight ? " table-panel__grid--laned" : ""}`}
        >
          <colgroup>
            {/* One declared width per column, so entering a cell does not
                resize the table — plus the row-number lane, and, when that
                lane is at the far right, a spacer that absorbs the slack.
                Without the spacer a full-width fixed layout would stretch the
                declared widths, which is the thing they exist to stop. */}
            <col className="table-panel__gutter-col" />
            {Array.from({ length: width }, (_, column) => (
              <col key={column} style={fitProse ? undefined : { width: `${columnWidth(column)}px` }} />
            ))}
            {laneRight && <col className="table-panel__spacer-col" />}
            {laneRight && <col className="table-panel__lane-col" />}
          </colgroup>
          <thead>
            <tr>
              <SelectAll onSelect={selectAll} />
              {Array.from({ length: width }, (_, column) => (
                <th
                  key={column}
                  scope="col"
                  className={`table-panel__head${
                    selection &&
                    containsCell(selection, selection.anchor.row, column)
                      ? " table-panel__head--active"
                      : ""
                  }${
                    selection?.kind === "columns" &&
                    columnsIn(selection).includes(column)
                      ? " table-panel__head--taken"
                      : ""
                  }`}
                  data-tip={`Column ${columnLabel(column)} — click to select it, drag for more, drag the edge to resize`}
                  onMouseDown={(event) => {
                    if (event.button === 2) return;
                    event.preventDefault();
                    dragging.current = "columns";
                    takeColumn(column, event.shiftKey);
                  }}
                  onMouseEnter={() => {
                    if (dragging.current === "columns")
                      extend(height - 1, column);
                  }}
                  onClick={(event) => takeColumn(column, event.shiftKey)}
                  onContextMenu={(event) =>
                    openMenu(event, 0, column, "column")
                  }
                >
                  {columnLabel(column)}
                  <Resizer
                    axis="column"
                    size={() => columnWidth(column)}
                    least={MIN_COLUMN_WIDTH}
                    onResize={(next) => widen(column, next)}
                    onTap={tapped(() => takeColumn(column, false))}
                    onFit={fitted(() => fitColumn(column))}
                  />
                </th>
              ))}
              {laneRight && <td className="table-panel__spacer" />}
              {laneRight && <SelectAll onSelect={selectAll} lane />}
            </tr>
          </thead>
          <tbody>
            {Array.from({ length: height }, (_, row) => {
              const isHeaderRow = header && row === 0;
              return (
                <tr
                  key={row}
                  // Per row rather than per cell: the cells read the measure
                  // from here, so one number governs a row and a row cannot
                  // end up two heights at once.
                  style={
                    {
                      "--table-row-height": fitProse
                        ? (size.heights?.[row] === undefined ? "auto" : `${size.heights[row]}px`)
                        : `${rowHeight(row)}px`,
                      "--table-cell-white-space":
                        size.heights?.[String(row)] === undefined ? "pre" : "pre-wrap",
                    } as React.CSSProperties
                  }
                >
                  {rowNumber(row)}
                  {Array.from({ length: width }, (_, column) => {
                    const label = cellLabel(column, row);
                    return (
                      <td
                        key={column}
                        className={
                          (isHeaderRow ? "table-panel__names " : "") + (fill.includes(row, column) ? "table-fill-preview" : "")
                        }
                        data-fill-row={row} data-fill-column={column}
                        onMouseEnter={() => fill.over(row, column)}
                        onContextMenu={(event) => openMenu(event, row, column)}
                        onMouseMove={() => pointRangeOver(row, column)}
                      >
                        <TableCell
                          row={row}
                          column={column}
                          value={cellAt(table, row, column)}
                          shown={shownAt(row, column)}
                          problem={computed.errors[label]}
                          selected={
                            active?.row === row && active.column === column
                          }
                          within={
                            selection !== null &&
                            !isSingleCell(selection) &&
                            containsCell(selection, row, column)
                          }
                          stepping={
                            steppingAt?.row === row &&
                            steppingAt.column === column
                          }
                          read={steppingReads.has(label)}
                          pointedAt={
                            pointer?.row === row && pointer.column === column
                          }
                          editing={
                            editing &&
                            active?.row === row &&
                            active.column === column
                          }
                          draft={draft}
                          editable={editable}
                          inputRef={inputRef}
                          selectedRef={selectedRef}
                          onDown={(event) => {
                            // A drag that begins on a cell sweeps a
                            // rectangle. In point mode the mousedown is
                            // swallowed instead, so the formula being typed
                            // does not lose focus and commit itself before
                            // the click can mean anything.
                            if (pointing) {
                              rangeAnchor.current = { row, column };
                              rangeDragged.current = false;
                              return;
                            }
                            dragging.current = "cells";
                            if (event.shiftKey) extend(row, column);
                            else select(row, column);
                          }}
                          onEnter={() => {
                            if (fill.dragging()) { fill.over(row, column); return; }
                            if (pointing && rangeAnchor.current) {
                              pointRangeOver(row, column);
                              return;
                            }
                            if (dragging.current === "cells")
                              extend(row, column);
                          }}
                          onSelect={(event) => {
                            if (rangeAnchor.current) {
                              const anchor = rangeAnchor.current;
                              rangeAnchor.current = null;
                              if (rangeDragged.current) {
                                pointRangeTo(anchor, row, column);
                                return;
                              }
                            }
                            if (pointTo(row, column)) return;
                            // Not a reference: the ordinary click it was, and
                            // the formula it interrupted ends here rather
                            // than through a blur that never came (the
                            // mousedown was swallowed to keep the input
                            // focused).
                            // The mousedown has already selected this cell
                            // — unless we were pointing, when it was
                            // swallowed to keep the input focused, and this
                            // click is the whole act.
                            // Doing it again is how a click stands on its own
                            // — a synthetic one, or a browser that sends no
                            // mousedown — and re-selecting the same cell
                            // costs nothing.
                            if (event.shiftKey) extend(row, column);
                            else select(row, column);
                          }}
                          onEdit={() => edit(row, column)}
                          onDraft={typed}
                          onPoint={pointKey}
                          onKeys={(event) =>
                            onSelectedKeyDown(event, { row, column })
                          }
                          onMove={(dRow, dColumn) =>
                            commitAndMove({ row, column }, dRow, dColumn)
                          }
                          onDone={stopEditing}
                          onCancel={() => setEditing(false)}
                        />
                        {editable && !editing && fill.at(row, column) && <TableFillHandle onStart={fill.start} range={fill.preview ? selectionLabel(fill.preview) : undefined} />}
                        {/* Grid edges resize; a stationary press selects the cell. */}
                        <Resizer
                          axis="row"
                          size={() => rowHeight(row)}
                          least={MIN_ROW_HEIGHT}
                          onResize={(next) => heighten(row, next)}
                          onTap={
                            editable
                              ? tapped(() => select(row, column))
                              : undefined
                          }
                          onFit={fitted(() => fitRow(row))}
                        />
                        <Resizer
                          axis="column"
                          size={() => columnWidth(column)}
                          least={MIN_COLUMN_WIDTH}
                          onResize={(next) => widen(column, next)}
                          onTap={
                            editable
                              ? tapped(() => select(row, column))
                              : undefined
                          }
                          onFit={fitted(() => fitColumn(column))}
                        />
                      </td>
                    );
                  })}
                  {laneRight && <td className="table-panel__spacer" />}
                  {laneRight && rowNumber(row, true)}
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      {menu && selection && (
        <ContextMenu
          x={menu.x}
          y={menu.y}
          className="table-menu"
          subject={`${rowsPhrase(selectedRows)}, ${columnsPhrase(selectedColumns)}`}
          items={tableMenuItems({
            rows: selectedRows,
            columns: selectedColumns,
            height,
            width,
          })}
          onPick={runMenu}
          onClose={() => setMenu(null)}
        />
      )}

      {/* Drag the bottom edge to give the table more of the document, or less.
          A dataset with two hundred rows and a dataset with four want very
          different amounts of room, and only the person reading it knows
          which one this is. */}
      {/* The drag starts from the height the grid actually HAS — the
          remembered one, or the nine rows it settled at — rather than from a
          measurement, so the first pixel of the drag does not jump. */}
      <HeightResizer
        height={gridHeight}
        onResize={(next) => resize({ ...size, height: next })}
      />

      {editable ? (
        <div className="table-panel__actions" role="toolbar" aria-label="Table">
          <button
            type="button"
            className="btn btn-small"
            onClick={() => apply(insertTableRow(table, height))}
            data-tip="Add a row at the bottom"
          >
            + Row
          </button>
          <button
            type="button"
            className="btn btn-small"
            onClick={() => apply(insertTableColumn(table, width))}
            data-tip="Add a column at the right"
          >
            + Column
          </button>
          <button
            type="button"
            className="btn btn-small"
            disabled={!canRemoveRows}
            // The mouse goes down on this button before the cell's input
            // blurs. Committing here means the rows that are about to be
            // removed do not first get written back from a stale draft.
            onMouseDown={(event) => event.preventDefault()}
            onClick={() => {
              if (!selection) return;
              apply(removeRows(table, selectedRows));
              setSelection(null);
              setEditing(false);
            }}
            data-tip={
              !selection
                ? "Select a cell or a row number first"
                : !canRemoveRows
                  ? "That is every row — a table with none left is not a table"
                  : selectedRows.length === 1
                    ? `Remove row ${selectedRows[0] + 1}`
                    : `Remove rows ${selectedRows[0] + 1}–${
                        selectedRows[selectedRows.length - 1] + 1
                      }`
            }
          >
            {selectedRows.length > 1
              ? `− ${selectedRows.length} Rows`
              : "− Row"}
          </button>
          <button
            type="button"
            className="btn btn-small"
            disabled={!canRemoveColumns}
            onMouseDown={(event) => event.preventDefault()}
            onClick={() => {
              if (!selection) return;
              apply(removeColumns(table, selectedColumns));
              setSelection(null);
              setEditing(false);
            }}
            data-tip={
              !selection
                ? "Select a cell or a column letter first"
                : !canRemoveColumns
                  ? "That is every column — a table with none left is not a table"
                  : selectedColumns.length === 1
                    ? `Remove column ${columnLabel(selectedColumns[0])}`
                    : `Remove columns ${columnLabel(selectedColumns[0])}–${columnLabel(
                        selectedColumns[selectedColumns.length - 1],
                      )}`
            }
          >
            {selectedColumns.length > 1
              ? `− ${selectedColumns.length} Columns`
              : "− Column"}
          </button>
          {formats !== undefined && <TableFormatControl owner={parent} table={table} selection={selection}
            draft={editing && active ? { ...active, value: draft } : null}
            sourceCurrent={sourceIsCurrent}
            onChange={(next, actor) => { const ok = apply(next, false, actor); if (ok) setEditing(false); return ok; }} />}
          {language && (
            <button
              type="button"
              className={`btn btn-small${showFormulas ? " btn-active" : ""}`}
              aria-pressed={showFormulas}
              onMouseDown={(event) => event.preventDefault()}
              onClick={() => setShowFormulas((on) => !on)}
              data-tip="Show the formulas instead of their values (Ctrl+`)"
            >
              Formulas
            </button>
          )}
          {language && hasFormulas && (
            <button
              type="button"
              className={`btn btn-small${debugging ? " btn-active" : ""}`}
              aria-pressed={debugging}
              onMouseDown={(event) => event.preventDefault()}
              onClick={() => setDebugging((on) => !on)}
              data-tip="Step through the formulas in the order they run"
            >
              Step
            </button>
          )}
          {language && (
            <span
              className="table-panel__lang muted"
              data-tip={`Formulas are ${language}`}
            >
              {language}
            </span>
          )}
          {/* The number that reports the size is the number you edit to
              change it. Pressing "+ Row" eleven times to get a 12-row table
              is how somebody ends up typing the CSV by hand instead. */}
          <button
            type="button"
            className="table-panel__size muted"
            aria-haspopup="dialog"
            aria-expanded={sizing}
            onMouseDown={(event) => event.preventDefault()}
            onClick={() => setSizing((on) => !on)}
            data-tip="Set how many rows and columns this table has"
          >
            {height} × {width}
          </button>
        </div>
      ) : (
        // A generated table's source is a document, and editing the output is
        // not the way to change it. Saying so beats a grid that silently
        // discards what you type.
        <p className="table-panel__readonly muted">
          Written by a document — edit it there.
        </p>
      )}

      {sizing && editable && (
        <TableSizeDialog
          owner={parent}
          rows={height}
          columns={width}
          filledRows={filledSize(table).rows}
          filledColumns={filledSize(table).columns}
          sourceCurrent={sourceIsCurrent}
          onCancel={() => setSizing(false)}
          onApply={(rows, columns, actor) => {
            if (!apply(resizeTable(table, rows, columns), true, actor)) return false;
            setSizing(false);
            // The old selection may name a cell that is no longer there.
            setSelection(null);
            setEditing(false);
            return true;
          }}
        />
      )}

      {/* Stepping through the formulas: which cell went when, and what it read
          when its turn came. The order is the host's contribution and the part
          a person cannot see by looking at the grid. */}
      {debugging && language && (
        <FormulaDebugger
          owner={parent}
          language={language}
          rows={table.rows}
          revision={source}
          onStep={onStep}
          onClose={() => setDebugging(false)}
        />
      )}

      {selectedProblem && (
        <p className="table-panel__note table-panel__note--bad" role="status">
          {active && cellLabel(active.column, active.row)}: {selectedProblem}
        </p>
      )}
      {computation === "pending" && <p className="table-panel__note muted">Formula values or notes may be from previous source; current calculation is pending.</p>}
      {operationError && <p className="table-panel__note table-panel__note--bad" role="alert">{operationError}</p>}
      {formulaNote && (
        <p className="table-panel__note muted" role="status">
          {formulaNote}
        </p>
      )}
    </div>
  );
}

</storycode:copy>
<a id="f-38d5395d7736-inAField"></a>
### `inAField`

Whether a clipboard event happened inside a text field.
The clipboard belongs to whatever has focus. An open cell and the formula
bar are both inputs, and copying the characters you selected inside one is
both what the browser already does and what was meant — a grid that
answered "here is the whole cell instead" would be taking the clipboard off
the person using it.

<storycode:copy id="f-38d5395d7736-inAField">
/**
 * Whether a clipboard event happened inside a text field.
 *
 * The clipboard belongs to whatever has focus. An open cell and the formula
 * bar are both inputs, and copying the characters you selected inside one is
 * both what the browser already does and what was meant — a grid that
 * answered "here is the whole cell instead" would be taking the clipboard off
 * the person using it.
 */
function inAField(event: React.ClipboardEvent): boolean {
  const target = event.target as HTMLElement | null;
  return target?.closest?.("input, textarea") != null;
}
</storycode:copy>

### Assemble `apps/web/src/components/TablePanel.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/TablePanel.tsx"><storycode:paste select="#f-38d5395d7736-TableLayout"/><storycode:paste select="#f-38d5395d7736-TablePanelProps"/><storycode:paste select="#f-38d5395d7736-times"/><storycode:paste select="#f-38d5395d7736-MIN_COLUMN_WIDTH"/><storycode:paste select="#f-38d5395d7736-MIN_ROW_HEIGHT"/><storycode:paste select="#f-38d5395d7736-MAX_COLUMN_WIDTH"/><storycode:paste select="#f-38d5395d7736-FIT_BORDER"/><storycode:paste select="#f-38d5395d7736-FIT_ANTI_CLIP"/><storycode:paste select="#f-38d5395d7736-ROW_HEIGHT"/><storycode:paste select="#f-38d5395d7736-HEAD_HEIGHT"/><storycode:paste select="#f-38d5395d7736-VISIBLE_ROWS"/><storycode:paste select="#f-38d5395d7736-MOVE_CLIPBOARD_TYPE"/><storycode:paste select="#f-38d5395d7736-clampTo"/><storycode:paste select="#f-38d5395d7736-TablePanel"/><storycode:paste select="#f-38d5395d7736-inAField"/></storycode:file>

</storycode:doc>
