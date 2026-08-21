// CSV, parsed and written back byte-for-byte where nothing changed.
//
// A table editor's whole job is to be a nicer way to edit a file that is
// still a file. That puts an unusual demand on the parser: it is not enough
// to read a table correctly, it has to WRITE one that a person would have
// typed, because the result lands in a document somebody else reviews in a
// diff. A parser that round-trips `a,b` as `"a","b"` turns a one-cell edit
// into a whole-file rewrite, and the diff stops being reviewable.
//
// So the rules here are RFC 4180's, plus one of our own:
//
//  - A field is quoted only when it HAS to be — when it holds the delimiter,
//    a quote, a newline, or leading/trailing space. Quoting everything is
//    valid and unreadable.
//  - The line ending the file already used is the line ending it keeps. A
//    Windows CSV edited here does not silently become a Unix one, which would
//    show up as every line changed.
//  - A trailing newline is preserved if it was there and not invented if it
//    was not.
//
// Ragged rows are kept ragged on the way in — a row with three fields where
// the header has four is a real thing that happens, and padding it on read
// would write four fields back out and claim the file said something it did
// not. The GRID pads for display; the file keeps what it had until a cell in
// that row is actually edited.

/** A parsed table: rows of fields, exactly as many as each row had. */
export interface Csv {
  rows: string[][];
  /** The delimiter that was found (or the one that was asked for). */
  delimiter: string;
  /** "\n" or "\r\n" — whichever the text used. */
  newline: string;
  /** Whether the text ended with a line break. */
  trailingNewline: boolean;
}

/** Delimiters worth guessing between. Tab and semicolon are what a European
 * spreadsheet and a TSV export produce, and both are common enough that
 * guessing wrong once is worse than looking. */
const CANDIDATES = [",", "\t", ";", "|"] as const;

/**
 * Which delimiter a text uses.
 *
 * Counted OUTSIDE quotes, on the first few lines, and the winner is the one
 * that appears the same number of times on every line — a real delimiter is
 * consistent per row, and a comma inside prose is not. Ties go to the comma,
 * because it is the one the format is named after.
 */
export function guessDelimiter(text: string): string {
  const sample = text.split(/\r?\n/).filter((l) => l.trim() !== "").slice(0, 10);
  if (sample.length === 0) return ",";
  let best = ",";
  let bestScore = -1;
  for (const candidate of CANDIDATES) {
    const counts = sample.map((line) => countOutsideQuotes(line, candidate));
    if (counts[0] === 0) continue;
    const consistent = counts.every((n) => n === counts[0]);
    // Consistency first, then how many fields it makes: `a,b;c,d` is two
    // semicolon-fields and three comma-fields, and the commas are the table.
    const score = (consistent ? 1000 : 0) + counts[0];
    if (score > bestScore) {
      bestScore = score;
      best = candidate;
    }
  }
  return bestScore < 0 ? "," : best;
}

function countOutsideQuotes(line: string, delimiter: string): number {
  let count = 0;
  let quoted = false;
  for (let i = 0; i < line.length; i++) {
    const ch = line[i];
    if (ch === '"') {
      // A doubled quote inside a quoted field is an escaped quote, not a
      // close followed by an open.
      if (quoted && line[i + 1] === '"') i++;
      else quoted = !quoted;
    } else if (!quoted && ch === delimiter) {
      count++;
    }
  }
  return count;
}

/** Parse `text` as CSV. Never throws: malformed input is read as best it can
 * be, because a table editor that refuses to open a file is useless exactly
 * when it is most needed. */
export function parseCsv(text: string, delimiter?: string): Csv {
  const sep = delimiter ?? guessDelimiter(text);
  const newline = /\r\n/.test(text) ? "\r\n" : "\n";
  const trailingNewline = /\r?\n$/.test(text);
  const rows: string[][] = [];
  let row: string[] = [];
  let field = "";
  let quoted = false;
  let i = 0;

  const endField = () => {
    row.push(field);
    field = "";
  };
  const endRow = () => {
    endField();
    rows.push(row);
    row = [];
  };

  while (i < text.length) {
    const ch = text[i];
    if (quoted) {
      if (ch === '"') {
        if (text[i + 1] === '"') {
          field += '"';
          i += 2;
          continue;
        }
        quoted = false;
        i++;
        continue;
      }
      field += ch;
      i++;
      continue;
    }
    if (ch === '"' && field === "") {
      // Only at the START of a field: a quote in the middle of `ab"cd` is a
      // literal quote, which is what a spreadsheet export produces.
      quoted = true;
      i++;
      continue;
    }
    if (ch === sep) {
      endField();
      i++;
      continue;
    }
    if (ch === "\r" && text[i + 1] === "\n") {
      endRow();
      i += 2;
      continue;
    }
    if (ch === "\n" || ch === "\r") {
      endRow();
      i++;
      continue;
    }
    field += ch;
    i++;
  }
  // An unterminated quoted field still ends the row it was in, rather than
  // losing it.
  if (field !== "" || row.length > 0 || !trailingNewline) endRow();

  // A trailing newline produces one empty trailing row; it is the line ending
  // of the last row, not a row of its own.
  if (trailingNewline && rows.length > 0) {
    const last = rows[rows.length - 1];
    if (last.length === 1 && last[0] === "") rows.pop();
  }
  return { rows, delimiter: sep, newline, trailingNewline };
}

/** Whether a field has to be quoted to survive a round trip. */
export function needsQuoting(field: string, delimiter: string): boolean {
  return (
    field.includes(delimiter) ||
    field.includes('"') ||
    field.includes("\n") ||
    field.includes("\r") ||
    field !== field.trim()
  );
}

/** One field, written the way a person would have typed it. */
export function writeField(field: string, delimiter: string): string {
  if (!needsQuoting(field, delimiter)) return field;
  return `"${field.replace(/"/g, '""')}"`;
}

/**
 * Write a table back out.
 *
 * Quotes only what has to be quoted, and keeps the line ending and the
 * trailing newline the file came with. The point is a reviewable diff: a
 * one-cell edit should be a one-line change.
 */
export function writeCsv(table: Csv): string {
  const body = table.rows
    .map((row) => row.map((field) => writeField(field, table.delimiter)).join(table.delimiter))
    .join(table.newline);
  return table.trailingNewline && body !== "" ? body + table.newline : body;
}

/** The widest row's length — how many columns the grid draws. */
export function columnCount(table: Pick<Csv, "rows">): number {
  return table.rows.reduce((n, row) => Math.max(n, row.length), 0);
}

/**
 * The value of one cell, padding for DISPLAY only.
 *
 * A row shorter than the header is a real thing that happens; the grid shows
 * it as empty cells, and the file keeps its ragged row until somebody edits
 * one of them.
 */
export function cellAt(table: Pick<Csv, "rows">, row: number, column: number): string {
  return table.rows[row]?.[column] ?? "";
}

/** The table with one cell replaced, padding that row only as far as it must. */
export function withCell(table: Csv, row: number, column: number, value: string): Csv {
  const rows = table.rows.map((r) => [...r]);
  while (rows.length <= row) rows.push([]);
  const target = rows[row];
  while (target.length <= column) target.push("");
  target[column] = value;
  return { ...table, rows };
}

/**
 * The table with a set of cells emptied.
 *
 * A cell PAST the end of its own row is skipped rather than padded: the file
 * keeps its ragged rows until somebody edits one, and clearing a cell that
 * was never there would write fields the file did not have.
 */
export function clearCells(table: Csv, cells: { row: number; column: number }[]): Csv {
  const rows = table.rows.map((r) => [...r]);
  for (const { row, column } of cells) {
    const target = rows[row];
    if (target && column < target.length) target[column] = "";
  }
  return { ...table, rows };
}

/** Remove several rows at once — a selection of them, from the toolbar.
 * Removed from the bottom up, so earlier removals do not shift the indices
 * of the ones still to come. */
export function removeRows(table: Csv, rows: number[]): Csv {
  return [...rows].sort((a, b) => b - a).reduce(removeRow, table);
}

/** Remove several columns at once, bottom-up for the same reason. */
export function removeColumns(table: Csv, columns: number[]): Csv {
  return [...columns].sort((a, b) => b - a).reduce(removeColumn, table);
}

/**
 * A block of cells written into the table with its top-left corner at
 * (`row`, `column`), growing the table as far as it has to.
 *
 * A paste is an instruction, so it grows: pasting four rows into the last row
 * of a table means the table now has three more. It does not, however, tidy
 * anything it did not touch — a ragged row two rows above stays ragged.
 */
export function pasteBlock(
  table: Csv,
  row: number,
  column: number,
  block: string[][],
): Csv {
  const rows = table.rows.map((r) => [...r]);
  block.forEach((fields, down) => {
    const at = row + down;
    while (rows.length <= at) rows.push([]);
    const target = rows[at];
    fields.forEach((field, across) => {
      const into = column + across;
      while (target.length <= into) target.push("");
      target[into] = field;
    });
  });
  return { ...table, rows };
}

/**
 * How much of the table actually holds something.
 *
 * Not the same as its shape: a table can be twelve rows tall with the last
 * four empty. What this answers is what a shrink would really take, so
 * "removes 4 rows" can be said only when there is something in them.
 */
export function filledSize(table: Pick<Csv, "rows">): { rows: number; columns: number } {
  let rows = 0;
  let columns = 0;
  table.rows.forEach((row, index) => {
    row.forEach((field, column) => {
      if (field.trim() === "") return;
      rows = Math.max(rows, index + 1);
      columns = Math.max(columns, column + 1);
    });
  });
  return { rows, columns };
}

/**
 * The table at exactly `rows` × `columns`.
 *
 * Grows with empty cells and trims from the end — the end, because a table is
 * read from the top left, and taking rows off the bottom is what "make it
 * smaller" means to the person who typed a smaller number.
 *
 * This is the one place a ragged row is squared up, and deliberately: padding
 * for DISPLAY would claim the file said something it did not, but an explicit
 * "make this table four columns wide" is an instruction rather than a guess.
 */
export function resizeTable(table: Csv, rows: number, columns: number): Csv {
  const wanted = Math.max(1, Math.floor(rows));
  const wide = Math.max(1, Math.floor(columns));
  const next: string[][] = [];
  for (let row = 0; row < wanted; row++) {
    const existing = table.rows[row] ?? [];
    const fields = existing.slice(0, wide);
    while (fields.length < wide) fields.push("");
    next.push(fields);
  }
  return { ...table, rows: next };
}

/** Insert an empty row at `at` (0-based; `rows.length` appends). */
export function insertRow(table: Csv, at: number): Csv {
  const width = columnCount(table);
  const rows = table.rows.map((r) => [...r]);
  rows.splice(Math.max(0, Math.min(at, rows.length)), 0, new Array(width).fill(""));
  return { ...table, rows };
}

/** Remove a row. Removing the last one leaves an empty table, not a broken
 * one. */
export function removeRow(table: Csv, at: number): Csv {
  const rows = table.rows.map((r) => [...r]);
  if (at >= 0 && at < rows.length) rows.splice(at, 1);
  return { ...table, rows };
}

/** Insert an empty column at `at` in every row. */
export function insertColumn(table: Csv, at: number): Csv {
  const width = columnCount(table);
  const index = Math.max(0, Math.min(at, width));
  const rows = table.rows.map((row) => {
    const next = [...row];
    // Pad only up to the insertion point: a ragged row stays as ragged as it
    // was on the right-hand side.
    while (next.length < index) next.push("");
    next.splice(index, 0, "");
    return next;
  });
  return { ...table, rows };
}

/** Remove a column from every row. */
export function removeColumn(table: Csv, at: number): Csv {
  const rows = table.rows.map((row) => {
    const next = [...row];
    if (at >= 0 && at < next.length) next.splice(at, 1);
    return next;
  });
  return { ...table, rows };
}

/**
 * A markdown table, for the weave.
 *
 * The JS twin of what `hick weave` writes, used by the browser demo and by
 * anything that needs the rendered form without a round trip. Pipes and
 * backslashes are escaped because a markdown table has no quoting of its own
 * — a cell containing `|` would silently become two cells.
 */
export function toMarkdown(table: Pick<Csv, "rows">, header = true): string {
  const width = columnCount(table);
  if (width === 0 || table.rows.length === 0) return "";
  const cell = (value: string) => value.replace(/\\/g, "\\\\").replace(/\|/g, "\\|");
  const line = (row: string[]) =>
    `| ${Array.from({ length: width }, (_, i) => cell(row[i] ?? "")).join(" | ")} |`;
  const out: string[] = [];
  const [first, ...rest] = table.rows;
  if (header) {
    out.push(line(first));
    out.push(`|${" --- |".repeat(width)}`);
    for (const row of rest) out.push(line(row));
  } else {
    // No header row means an empty one: markdown has no table without a
    // header, and inventing column names would be inventing content.
    out.push(`|${"  |".repeat(width)}`);
    out.push(`|${" --- |".repeat(width)}`);
    for (const row of table.rows) out.push(line(row));
  }
  return out.join("\n");
}

/**
 * Whether a fence's info string names a table.
 *
 * `csv` and `tsv` only. Not `data`, not `txt`: a fence gets a GRID instead of
 * the "make it a cell" converter on the strength of this, and guessing wrong
 * takes away the converter from something that wanted it.
 */
export function isTabularFence(info: string): boolean {
  const language = info.trim().split(/\s+/)[0]?.toLowerCase() ?? "";
  return language === "csv" || language === "tsv";
}
