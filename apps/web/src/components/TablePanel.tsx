// A table, edited as a grid.
//
// The point of this component is that the file underneath stays a CSV file
// somebody else will read in a diff. So every edit goes back through
// `writeCsv`, which quotes only what has to be quoted and keeps the line
// ending the file came with — a one-cell edit is a one-line change, and the
// review stays reviewable. See lib/csv.ts.
//
// Everything above that is spreadsheet convention, on purpose. The people who
// will use this have spent years in Excel, and every place this differs is a
// place they are wrong about what will happen. So: A1 labels down the side and
// across the top, a name box and a formula bar, click to SELECT and a second
// act to edit, drag to select a rectangle, a letter or a number to take a
// whole column or row, Enter to commit and drop a line, and — while a formula
// is open — a click on another cell writes its reference into what you are
// typing. Ctrl+` shows the formulas instead of their values.
//
// The select/edit split is the load-bearing one, and it is not decoration. A
// cell that goes straight into edit mode has no state in which the toolbar can
// act on it — pressing "− Row" blurs the input, which clears the cursor, which
// disables the button before the click lands. Selection that survives losing
// focus is what makes a toolbar possible at all, and it is what Excel does.
//
// Selection is a RANGE — an anchor and a reach, in lib/tableSelection.ts —
// rather than a cursor, because a toolbar that can only ever remove one row
// is a toolbar people work around by clicking four times. The anchor is still
// a single cell and still does everything a cursor did: it is what the
// formula bar edits and what a keystroke replaces.
//
// The grid is `table-layout: fixed` with declared column widths for a related
// reason: a cell is a span until you enter it and an input while you are in
// it, and under automatic layout those two measure differently, so entering a
// cell resized the whole table under the pointer.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { api } from "../api/client";
import type { FormulaStep } from "../api/types";
import { cellLabel, columnLabel, isFormula, parseCellLabel } from "../lib/cellRef";
import { pointAt, type PointedAt } from "../lib/formulaPoint";
import { parseClipboardTable, toHtmlTable, toTabSeparated } from "../lib/tableClipboard";
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

import { FormulaDebugger } from "./FormulaDebugger";
import { TableSizeDialog } from "./TableSizeDialog";

import {
  cellAt,
  clearCells,
  columnCount,
  filledSize,
  insertColumn,
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
  /** The grid's visible height in pixels. Absent means the default measure. */
  height?: number;
  /** Column widths in pixels, keyed by column index. */
  widths?: Record<string, number>;
  /** Row heights in pixels, keyed by row index. Absent rows are the default
   * measure, which is what nearly every row is. */
  heights?: Record<string, number>;
}

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
  /** How big this table was left last time. Seeded once — the panel owns it
   * from then on and reports changes through `onLayout`. */
  layout?: TableLayout;
  /** Remember a new size. Absent means the size is this session's only. */
  onLayout?: (next: TableLayout) => void;
  /** Also draw the row numbers in a lane at the FAR RIGHT of the grid, where
   * they line up with this editor's line-number rail. A rendered table is a
   * fold, so that rail has one number for the whole block and cannot label
   * the rows itself.
   *
   * As well as, never instead of: the numbers beside the first column are
   * where a spreadsheet puts them and where a hand goes to grab a row, and
   * the lane on the right is about lining up with the editor's rail. Those
   * are two different jobs and dropping either one to do the other was the
   * wrong trade. */
  laneRight?: boolean;
}

/** A column's width when nobody has dragged it. Wide enough for a short
 * label or a number, narrow enough that eight columns fit without scrolling. */
const DEFAULT_COLUMN_WIDTH = 104;
/** Narrow enough to tuck a column out of the way, wide enough to grab again. */
const MIN_COLUMN_WIDTH = 40;
/** Short enough to squeeze a row down to a line, tall enough to grab again. */
const MIN_ROW_HEIGHT = 16;
/** The ceilings, matching what `readTableLayout` will store: a fit is driven
 * by content, and one enormous cell must not make a column nobody can scroll
 * past. */
const MAX_COLUMN_WIDTH = 2000;
const MAX_ROW_HEIGHT = 600;
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
const FIT_ANTI_CLIP = 1;
/** How far the pointer has to travel before a press on a grid line is a DRAG
 * rather than a click.
 *
 * This is what stops the handles from being dead zones. A grid line runs
 * along the edge of a cell somebody also wants to click, and a five-pixel
 * strip that swallowed clicks would make the bottom of every row unselectable
 * — so a press that does not move falls through to the cell underneath. */
const DRAG_THRESHOLD = 3;
/** Below this the grid is the furniture and none of the data. */
const MIN_GRID_HEIGHT = 64;
/**
 * How tall one body row is, in pixels, and the letter row above them.
 *
 * Declared rather than measured, for the same reason the column widths are:
 * a cell is a span until you enter it and an input while you are in it, and
 * measuring would make entering one resize the grid. Declaring it also makes
 * "nine rows" an arithmetic fact rather than a hope — the number is handed to
 * the stylesheet as a custom property, so the rows really are this tall.
 */
const ROW_HEIGHT = 24;
const HEAD_HEIGHT = 22;
/**
 * How many rows a table shows before it scrolls instead of growing.
 *
 * A table in a document is a paragraph (see the guarantee). A hundred-row
 * dataset that pushes the prose after it off the screen has stopped being one,
 * and the reader who wanted the tenth row wanted it *next to* the sentence
 * about it. So past nine rows the grid keeps its size and scrolls, and the
 * person who does want to see forty at once drags the bottom edge — which is
 * a decision about this table, remembered with the rest of its layout.
 */
const VISIBLE_ROWS = 9;

const clampTo = (value: number, least: number, most: number) =>
  Math.max(least, Math.min(most, Math.round(value)));

/**
 * How much room these cells' contents would take if nothing constrained them.
 *
 * The measurement an auto-fit needs, and NOT what `scrollWidth` answers.
 * A cell fills its column and its row, so its scroll size is the larger of
 * its content and its box — which means it reports the size the cell already
 * has whenever the text is smaller than it, and a fit built on it can only
 * ever grow. That reads as "double-clicking anything expands it slightly",
 * because the slack is the only thing that changed.
 *
 * So the constraint is lifted for the length of the measurement: `max-content`
 * on the axis being asked about, the box read back, the inline style put back.
 * It is the same element with the same font and the same padding — only the
 * width or height it was being held to is gone — so the answer is exact by
 * construction rather than by a second copy of the cell's styling.
 *
 * Fractional, because text is: the caller rounds up once at the end rather
 * than losing a fraction of a pixel per cell.
 */
function intrinsic(cells: Iterable<HTMLElement>, axis: "width" | "height"): number {
  let most = 0;
  for (const cell of cells) {
    const held = cell.style[axis];
    cell.style[axis] = "max-content";
    most = Math.max(most, cell.getBoundingClientRect()[axis]);
    // Restored immediately, so a fit never leaves a cell laid out differently
    // from the ones beside it.
    cell.style[axis] = held;
  }
  return Math.ceil(most);
}

export function TablePanel({
  source,
  header = true,
  delimiter,
  onChange,
  language,
  layout,
  onLayout,
  laneRight = false,
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

  // Seeded from the remembered layout and owned from then on: the prop is a
  // starting point, not a controlled value, so a drag does not have to make a
  // round trip through the workspace's state on every pointermove.
  const [size, setSize] = useState<TableLayout>(() => layout ?? {});
  const resize = (next: TableLayout) => {
    setSize(next);
    onLayout?.(next);
  };
  const columnWidth = (column: number) => size.widths?.[String(column)] ?? DEFAULT_COLUMN_WIDTH;
  const widen = (column: number, to: number) =>
    resize({ ...size, widths: { ...size.widths, [String(column)]: to } });
  const heighten = (row: number, to: number) =>
    resize({ ...size, heights: { ...size.heights, [String(row)]: to } });
  /** How tall one row is. Nearly every row is the default measure; a row that
   * was dragged taller is remembered by index alongside the widths. */
  const rowHeight = (row: number) => size.heights?.[String(row)] ?? ROW_HEIGHT;
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

  const gridRef = useRef<HTMLTableElement | null>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);
  const barRef = useRef<HTMLInputElement | null>(null);
  const selectedRef = useRef<HTMLElement | null>(null);
  // Which axis a drag is sweeping: cells, whole rows, or whole columns. Null
  // when nothing is being dragged.
  const dragging = useRef<Selection["kind"] | null>(null);
  // Where the last pointed-at reference landed in the draft, so the next
  // click replaces it instead of piling up beside it. See lib/formulaPoint.ts.
  const pointed = useRef<PointedAt | null>(null);
  // Where to leave the caret after a reference is written in. Applied in an
  // effect because the input has not re-rendered with the new text yet.
  const [caretWanted, setCaretWanted] = useState<number | null>(null);

  const activeInput = () => (entryFrom === "bar" ? barRef.current : inputRef.current);

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
    (next: Csv) => {
      const text = writeCsv(next);
      // A write that changes nothing still marks the document dirty and still
      // lands in the undo history, which is how pressing Delete on already
      // empty cells ends up as a commit somebody has to explain.
      if (text !== source) onChange?.(text);
    },
    [onChange, source],
  );

  /** Write the draft into the cell it belongs to, if it changed anything. */
  const commit = useCallback(
    (at: CellAt, value: string) => {
      if (cellAt(table, at.row, at.column) === value) return;
      apply(withCell(table, at.row, at.column, value));
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
      current ? extendTo(current, row, column, height, width) : cellSelection(row, column),
    );
  };

  /** Start typing into a cell. `seed` replaces its contents, as typing over a
   * selected cell does in every spreadsheet; absent, the cell's own text is
   * the starting point, which is what F2 and a double-click mean. */
  const edit = (row: number, column: number, seed?: string, from: "grid" | "bar" = "grid") => {
    if (!editable) return;
    setSelection(cellSelection(row, column));
    setDraft(seed ?? cellAt(table, row, column));
    setEntryFrom(from);
    setEditing(true);
    // A new edit has pointed at nothing yet; the last edit's reference is not
    // this one's to replace.
    pointed.current = null;
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

  /**
   * A double-click on a grid line: the column, or the row, at the smallest
   * measure that still fits what is in it.
   *
   * This is the ONE place the grid measures rather than declares, and it has
   * to be: "what will still fit" is a question about rendered text in a font
   * this component cannot know. It asks the cells, which are the only things
   * that know — each is clipped to its column, so its `scrollWidth` is the
   * width its text WOULD have taken. The answer is then written back as a
   * declared number like any other, so everything downstream (the nine-row
   * height, entering a cell without the table moving) is unaffected.
   *
   * A column with nothing in it fits to the minimum, which is the honest
   * answer to "how much room does nothing need".
   */
  const fitColumn = (column: number) => {
    const cells = gridRef.current?.querySelectorAll<HTMLElement>(`[data-column="${column}"]`);
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

  /** The same for a row. A cell holding a newline — which a paste from a web
   * page can produce — is several lines tall, and this is what makes room for
   * it without anybody counting them. */
  const fitRow = (row: number) => {
    const cells = gridRef.current?.querySelectorAll<HTMLElement>(`[data-row="${row}"]`);
    if (!cells || cells.length === 0) return;
    heighten(
      row,
      clampTo(intrinsic(cells, "height") + FIT_BORDER, MIN_ROW_HEIGHT, MAX_ROW_HEIGHT),
    );
  };

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
  const pointing = editable && editing && language !== undefined && isFormula(draft);

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
    const written = pointAt(draft, caret, cellLabel(column, row), pointed.current);
    if (!written) return false;
    setDraft(written.text);
    pointed.current = written.pointed;
    setCaretWanted(written.caret);
    return true;
  };

  /**
   * The selected rectangle, as cells.
   *
   * What goes on the clipboard, and what "− Row" would take: the same
   * rectangle the grid is drawing, read out of the table rather than out of
   * anything the clipboard has to be told about separately.
   */
  const blockOf = (of: Selection): string[][] =>
    rowsIn(of).map((row) => columnsIn(of).map((column) => cellAt(table, row, column)));

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
  const copyOut = (clipboard: DataTransfer | null | undefined): boolean => {
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
  const pasteIn = (clipboard: DataTransfer | null | undefined): boolean => {
    if (!editable || !selection || !clipboard) return false;
    const block = parseClipboardTable(
      clipboard.getData("text/html"),
      clipboard.getData("text/plain"),
    );
    if (!block || block.length === 0) return false;
    const at = selection.anchor;
    apply(pasteBlock(table, at.row, at.column, block));
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

  /**
   * What a cell shows when it is not being edited.
   *
   * A formula shows its VALUE and reveals its source when you enter it —
   * which is what every spreadsheet does, and the only arrangement in which
   * a table of formulas is readable. A formula with no value yet (nothing
   * evaluated, or an error) shows its own text, because a blank cell would
   * be a lie about there being nothing there. Ctrl+` shows every formula's
   * text at once, which is how a sheet is read rather than used.
   */
  const shownAt = (row: number, column: number): string => {
    const raw = cellAt(table, row, column);
    if (!isFormula(raw) || showFormulas) return raw;
    return computed.values[cellLabel(column, row)] ?? raw;
  };

  /** Keys handled by a selected-but-not-editing cell: the spreadsheet ones. */
  const onSelectedKeyDown = (event: React.KeyboardEvent, at: CellAt) => {
    const key = event.key;
    if (key === "ArrowUp" || key === "ArrowDown" || key === "ArrowLeft" || key === "ArrowRight") {
      event.preventDefault();
      // Shift REACHES rather than moves, which is how a range is made without
      // a mouse — and the only way to make one on a keyboard at all.
      const from = event.shiftKey && selection ? selection.focus : at;
      const row = from.row + (key === "ArrowDown" ? 1 : key === "ArrowUp" ? -1 : 0);
      const column = from.column + (key === "ArrowRight" ? 1 : key === "ArrowLeft" ? -1 : 0);
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
  const canRemoveColumns = selectedColumns.length > 0 && selectedColumns.length < width;

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
  const onStep = useCallback((step: FormulaStep | null) => setStepping(step), []);

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
        event.preventDefault();
        dragging.current = "rows";
        takeRow(row, event.shiftKey);
      }}
      onMouseEnter={() => {
        if (dragging.current === "rows") extend(row, width - 1);
      }}
      onClick={(event) => takeRow(row, event.shiftKey)}
    >
      {row + 1}
      <Resizer
        axis="row"
        size={rowHeight(row)}
        least={MIN_ROW_HEIGHT}
        onResize={(next) => heighten(row, next)}
        onTap={tapped(() => takeRow(row, false))}
        onFit={fitted(() => fitRow(row))}
      />
    </th>
  );

  return (
    <div
      className="table-panel"
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
      // The clipboard, as EVENTS rather than as `navigator.clipboard`: the
      // event carries what was copied in every flavour it was copied in, needs
      // no permission prompt, and is the only way to read the `text/html` a
      // spreadsheet puts there. These fire on whatever inside the grid has
      // focus, which is the selected cell, and bubble to here.
      onCopy={(event) => {
        if (inAField(event)) return;
        if (copyOut(event.clipboardData)) event.preventDefault();
      }}
      onCut={(event) => {
        if (inAField(event) || !editable || !selection) return;
        if (!copyOut(event.clipboardData)) return;
        event.preventDefault();
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
            edit(active.row, active.column, event.target.value, "bar");
          }}
          onBlur={stopEditing}
          onKeyDown={(event) => {
            if (!active) return;
            if (event.key === "Enter") {
              event.preventDefault();
              commitAndMove(active, event.shiftKey ? -1 : 1, 0);
            } else if (event.key === "Escape") {
              event.preventDefault();
              setDraft(selectedRaw);
              setEditing(false);
            }
          }}
        />
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
              <col key={column} style={{ width: `${columnWidth(column)}px` }} />
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
                    selection && containsCell(selection, selection.anchor.row, column)
                      ? " table-panel__head--active"
                      : ""
                  }${
                    selection?.kind === "columns" && columnsIn(selection).includes(column)
                      ? " table-panel__head--taken"
                      : ""
                  }`}
                  data-tip={`Column ${columnLabel(column)} — click to select it, drag for more, drag the edge to resize`}
                  onMouseDown={(event) => {
                    event.preventDefault();
                    dragging.current = "columns";
                    takeColumn(column, event.shiftKey);
                  }}
                  onMouseEnter={() => {
                    if (dragging.current === "columns") extend(height - 1, column);
                  }}
                  onClick={(event) => takeColumn(column, event.shiftKey)}
                >
                  {columnLabel(column)}
                  <Resizer
                    axis="column"
                    size={columnWidth(column)}
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
                  style={{ "--table-row-height": `${rowHeight(row)}px` } as React.CSSProperties}
                >
                  {rowNumber(row)}
                  {Array.from({ length: width }, (_, column) => {
                    const label = cellLabel(column, row);
                    return (
                      <td key={column} className={isHeaderRow ? "table-panel__names" : undefined}>
                        <Cell
                          row={row}
                          column={column}
                          value={cellAt(table, row, column)}
                          shown={shownAt(row, column)}
                          problem={computed.errors[label]}
                          selected={active?.row === row && active.column === column}
                          within={
                            selection !== null &&
                            !isSingleCell(selection) &&
                            containsCell(selection, row, column)
                          }
                          stepping={steppingAt?.row === row && steppingAt.column === column}
                          read={steppingReads.has(label)}
                          editing={editing && active?.row === row && active.column === column}
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
                            if (pointing) return;
                            dragging.current = "cells";
                            if (event.shiftKey) extend(row, column);
                            else select(row, column);
                          }}
                          onEnter={() => {
                            if (dragging.current === "cells") extend(row, column);
                          }}
                          onSelect={(event) => {
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
                          onDraft={setDraft}
                          onKeys={(event) => onSelectedKeyDown(event, { row, column })}
                          onMove={(dRow, dColumn) => commitAndMove({ row, column }, dRow, dColumn)}
                          onDone={stopEditing}
                          onCancel={() => setEditing(false)}
                        />
                        {/* The two grid lines this cell owns: its bottom edge
                            and its right edge. After the cell in the DOM, so
                            they sit above it and can be grabbed; a press that
                            does not move falls back through to selecting the
                            cell, so neither edge is a dead zone. */}
                        <Resizer
                          axis="row"
                          size={rowHeight(row)}
                          least={MIN_ROW_HEIGHT}
                          onResize={(next) => heighten(row, next)}
                          onTap={editable ? tapped(() => select(row, column)) : undefined}
                          onFit={fitted(() => fitRow(row))}
                        />
                        <Resizer
                          axis="column"
                          size={columnWidth(column)}
                          least={MIN_COLUMN_WIDTH}
                          onResize={(next) => widen(column, next)}
                          onTap={editable ? tapped(() => select(row, column)) : undefined}
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

      {/* Drag the bottom edge to give the table more of the document, or less.
          A dataset with two hundred rows and a dataset with four want very
          different amounts of room, and only the person reading it knows
          which one this is. */}
      {/* The drag starts from the height the grid actually HAS — the
          remembered one, or the nine rows it settled at — rather than from a
          measurement, so the first pixel of the drag does not jump. */}
      <HeightResizer height={gridHeight} onResize={(next) => resize({ ...size, height: next })} />

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
            {selectedRows.length > 1 ? `− ${selectedRows.length} Rows` : "− Row"}
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
            {selectedColumns.length > 1 ? `− ${selectedColumns.length} Columns` : "− Column"}
          </button>
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
            <span className="table-panel__lang muted" data-tip={`Formulas are ${language}`}>
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
        <p className="table-panel__readonly muted">Written by a document — edit it there.</p>
      )}

      {sizing && editable && (
        <TableSizeDialog
          rows={height}
          columns={width}
          filledRows={filledSize(table).rows}
          filledColumns={filledSize(table).columns}
          onCancel={() => setSizing(false)}
          onApply={(rows, columns) => {
            apply(resizeTable(table, rows, columns));
            setSizing(false);
            // The old selection may name a cell that is no longer there.
            setSelection(null);
            setEditing(false);
          }}
        />
      )}

      {/* Stepping through the formulas: which cell went when, and what it read
          when its turn came. The order is the host's contribution and the part
          a person cannot see by looking at the grid. */}
      {debugging && language && (
        <FormulaDebugger
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
      {formulaNote && (
        <p className="table-panel__note muted" role="status">
          {formulaNote}
        </p>
      )}
    </div>
  );
}

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

/** The box above the row numbers: click it for the whole table, as every
 * spreadsheet's corner does. */
function SelectAll({ onSelect, lane = false }: { onSelect: () => void; lane?: boolean }) {
  return (
    <th
      className={`table-panel__corner${lane ? " table-panel__corner--lane" : ""}`}
      scope="col"
      // The lane's corner is the same box a second time, at the other end of
      // the same row. It keeps the click, because a target that looks like
      // one should behave like one — and loses the name, because two things
      // called "Select the whole table" is a screen reader reading the same
      // control twice.
      aria-label={lane ? undefined : "Select the whole table"}
      aria-hidden={lane ? true : undefined}
      data-tip="Select the whole table"
      onMouseDown={(event) => {
        event.preventDefault();
        onSelect();
      }}
      onClick={onSelect}
    />
  );
}

/**
 * The drag handle on a grid line.
 *
 * Every line in the table is one — the right edge of a column and the bottom
 * edge of a row, in the header, in the row numbers, and in every cell. A
 * spreadsheet only puts them in its headers, which is fine when the headers
 * are always on screen and irritating when the line you want to move is the
 * one your pointer is already next to.
 *
 * Two things make that affordable rather than annoying:
 *
 *   - **A press that does not move is a click.** The strip is five pixels
 *     along the edge of a cell people also want to select, so it hands the
 *     press back rather than swallowing it. Without this the bottom of every
 *     row would be unselectable.
 *   - **The measure is a number this component owns**, not one it reads off
 *     the screen — the same reason the widths are declared (see the note at
 *     the top). A handle in a cell has no idea how wide its column is; it is
 *     told, and it reports a new number back.
 *
 * Sizes live in the component and not in the document: how wide a column
 * looks is not something the CSV says, and writing it there would put
 * presentation into a file whose whole justification is that a script can
 * read it.
 */
function Resizer({
  axis,
  size,
  least,
  onResize,
  onTap,
  onFit,
}: {
  axis: "column" | "row";
  /** What it is now, in pixels — the number the drag starts from. */
  size: number;
  least: number;
  onResize: (size: number) => void;
  /** A press that never became a drag. Absent means such a press does
   * nothing, which is what a read-only table wants. */
  onTap?: () => void;
  /** A double-click: the smallest measure that still fits the content. The
   * gesture every spreadsheet has on this exact target. */
  onFit?: () => void;
}) {
  const along = (event: { clientX: number; clientY: number }) =>
    axis === "column" ? event.clientX : event.clientY;

  const onMouseDown = (event: React.MouseEvent) => {
    // Without these the press also selects the cell underneath, so a drag
    // that started on the line would move the selection as a side effect.
    event.preventDefault();
    event.stopPropagation();
    const from = along(event);
    let dragged = false;
    const move = (e: MouseEvent) => {
      const to = along(e);
      if (!dragged && Math.abs(to - from) < DRAG_THRESHOLD) return;
      dragged = true;
      onResize(Math.max(least, Math.round(size + (to - from))));
    };
    const up = (e: MouseEvent) => {
      window.removeEventListener("mousemove", move);
      window.removeEventListener("mouseup", up);
      // `detail` is the click count, so the second press of a double-click is
      // 2 and does not tap. The FIRST one still does — nothing can know a
      // second is coming — which is why the fit gives the selection back.
      if (!dragged && e.detail <= 1) onTap?.();
    };
    window.addEventListener("mousemove", move);
    window.addEventListener("mouseup", up);
  };

  return (
    <span
      className={`table-panel__resizer table-panel__resizer--${axis}`}
      role="presentation"
      data-testid={`${axis}-resizer`}
      onMouseDown={onMouseDown}
      onClick={(event) => event.stopPropagation()}
      onDoubleClick={(event) => {
        // Stopped so the second click does not also reach the cell behind
        // this strip, where a double-click means "open this for editing".
        event.stopPropagation();
        onFit?.();
      }}
    />
  );
}

/**
 * The drag bar along the bottom of the grid.
 *
 * Measured against the scroller it sits under rather than against a remembered
 * number, so the first drag starts from whatever height the table currently
 * has instead of jumping to a default.
 */
function HeightResizer({
  height,
  onResize,
}: {
  height?: number;
  onResize: (height: number) => void;
}) {
  const bar = useRef<HTMLDivElement | null>(null);

  // Mouse events rather than pointer events, matching `Resizer` beside
  // it: two drag handles on one component that behave differently is a thing
  // somebody eventually trips over.
  const onMouseDown = (event: React.MouseEvent) => {
    event.preventDefault();
    const scroller = bar.current?.previousElementSibling as HTMLElement | null;
    const startHeight = height ?? scroller?.getBoundingClientRect().height ?? MIN_GRID_HEIGHT;
    const startY = event.clientY;
    const move = (e: MouseEvent) =>
      onResize(Math.max(MIN_GRID_HEIGHT, Math.round(startHeight + (e.clientY - startY))));
    const up = () => {
      window.removeEventListener("mousemove", move);
      window.removeEventListener("mouseup", up);
    };
    window.addEventListener("mousemove", move);
    window.addEventListener("mouseup", up);
  };

  return (
    <div
      ref={bar}
      className="table-panel__height"
      role="separator"
      aria-label="Table height"
      aria-orientation="horizontal"
      data-testid="height-resizer"
      data-tip="Drag to see more rows at a time, or fewer"
      onMouseDown={onMouseDown}
    />
  );
}

function Cell({
  row,
  column,
  value,
  shown,
  problem,
  selected,
  within,
  stepping,
  read,
  editing,
  draft,
  editable,
  inputRef,
  selectedRef,
  onDown,
  onEnter,
  onSelect,
  onEdit,
  onDraft,
  onKeys,
  onMove,
  onDone,
  onCancel,
}: {
  /** Where it is, drawn onto the element so an auto-fit can find it. */
  row: number;
  column: number;
  /** The cell's own text — a formula, or a literal. */
  value: string;
  /** What to display: a formula's computed value, or its text. */
  shown: string;
  /** What the language said, when this formula did not evaluate. */
  problem?: string;
  /** The active cell, which stays active when focus goes elsewhere. */
  selected: boolean;
  /** Inside the selected rectangle, but not the anchor. */
  within: boolean;
  /** The cell whose turn it is, while somebody is stepping the formulas. */
  stepping: boolean;
  /** A cell the stepped-to formula read. */
  read: boolean;
  editing: boolean;
  draft: string;
  editable: boolean;
  inputRef: React.MutableRefObject<HTMLInputElement | null>;
  selectedRef: React.MutableRefObject<HTMLElement | null>;
  onDown: (event: React.MouseEvent) => void;
  onEnter: () => void;
  onSelect: (event: React.MouseEvent) => void;
  onEdit: () => void;
  onDraft: (value: string) => void;
  onKeys: (event: React.KeyboardEvent) => void;
  onMove: (dRow: number, dColumn: number) => void;
  onDone: () => void;
  onCancel: () => void;
}) {
  if (!editing) {
    return (
      <span
        ref={(node) => {
          if (selected) selectedRef.current = node;
        }}
        // Where this cell is, so an auto-fit can find every cell in a column
        // or a row and ask it how much room its own text needs. The only
        // thing in the grid that is MEASURED rather than declared — see
        // `fitColumn`.
        data-row={row}
        data-column={column}
        className={
          "table-panel__cell" +
          (problem ? " table-panel__cell--bad" : "") +
          (value !== shown ? " table-panel__cell--computed" : "") +
          (selected ? " table-panel__cell--selected" : "") +
          (within ? " table-panel__cell--within" : "") +
          (stepping ? " table-panel__cell--stepping" : "") +
          (read ? " table-panel__cell--read" : "")
        }
        role={editable ? "gridcell" : undefined}
        tabIndex={editable ? 0 : undefined}
        // A computed cell's hover shows the formula behind it; a broken one
        // shows the LANGUAGE's own message, because `#VALUE!` throws away the
        // only part the author can act on.
        data-tip={problem ?? (value !== shown ? value : undefined)}
        // One click selects and a second act opens, which is the spreadsheet
        // arrangement — and the one that leaves the toolbar something to act
        // on once focus has moved to a button. The mousedown is what selects,
        // because that is also where a drag begins; the click is left for the
        // one case where it means something else, which is pointing at a cell
        // from inside an open formula.
        onMouseDown={
          editable
            ? (event) => {
                // ALWAYS prevented, for two reasons that happen to want the
                // same thing. A mousedown that runs its course starts a text
                // selection, so sweeping a rectangle also sweeps the page's
                // prose into a blue smear — `user-select: none` alone would
                // not stop a drag that leaves the grid. And while a formula
                // is open it would move focus out of the input, committing
                // the edit before the click could be read as a reference.
                //
                // What the default would otherwise have given us is focus,
                // and the effect above puts that back on whichever cell ends
                // up selected.
                event.preventDefault();
                onDown(event);
              }
            : undefined
        }
        onMouseEnter={editable ? onEnter : undefined}
        onClick={editable ? onSelect : undefined}
        onDoubleClick={editable ? onEdit : undefined}
        onKeyDown={editable ? onKeys : undefined}
      >
        {/* A non-breaking space, so an empty cell is still a target with a
            height. A zero-height row is a row nobody can click into. */}
        {shown === "" ? " " : shown}
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
          // Escape ABANDONS, which is the whole difference between it and
          // clicking away. An Escape that saved would be the one chord in a
          // spreadsheet that means the opposite of what it means everywhere.
          event.preventDefault();
          onCancel();
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
