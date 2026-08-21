// The grid's half of the clipboard.
//
// A table people can select a rectangle in is a table people will try to copy
// out of and paste into, and the other end of that is almost never this app:
// it is Excel, or Google Sheets, or a `<table>` on a web page, or a block of
// CSV in a chat message. So the formats here are the ones those programs
// already speak, and the rule is to WRITE what everything reads and READ
// whatever arrives.
//
// Writing is two flavours of the same rectangle, because a clipboard carries
// several and the receiving program picks:
//
//   - `text/plain` as TAB-separated lines. This is what a spreadsheet writes
//     and what one expects to be given; comma-separated text pasted into
//     Excel lands in a single column, which is the single most annoying thing
//     a clipboard can do.
//   - `text/html` as a real `<table>`, which is what Excel, Word and Sheets
//     read when they want structure rather than text.
//
// Reading prefers the HTML when it holds a table, because that is the flavour
// that survived a cell containing a newline; otherwise the plain text is
// parsed as delimited data, with the delimiter guessed exactly as an opened
// file's is (lib/csv.ts).

import { parseCsv } from "./csv";

/** A rectangle of cells, row-major. */
export type Block = string[][];

/**
 * The rectangle as `text/plain`.
 *
 * A field is quoted only when it has to be — when it holds a tab, a newline,
 * or a quote — which is the rule Excel writes by, and the rule that keeps an
 * ordinary table pasteable into anything at all.
 */
export function toTabSeparated(block: Block): string {
  return block
    .map((row) =>
      row
        .map((field) =>
          /[\t\r\n"]/.test(field) ? `"${field.replace(/"/g, '""')}"` : field,
        )
        .join("\t"),
    )
    // CRLF, which is what every spreadsheet puts between rows and what some of
    // them still need to see.
    .join("\r\n");
}

/** The rectangle as `text/html`: a real table, so a program that wants
 * structure gets structure. */
export function toHtmlTable(block: Block): string {
  const escape = (text: string) =>
    text
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      // A newline inside a cell is the reason the HTML flavour exists; keeping
      // it as a `<br>` is what makes it survive the trip.
      .replace(/\n/g, "<br>");
  const rows = block
    .map((row) => `<tr>${row.map((field) => `<td>${escape(field)}</td>`).join("")}</tr>`)
    .join("");
  return `<table>${rows}</table>`;
}

/**
 * The first table in a fragment of HTML, or null if there is not one.
 *
 * `colspan` pads with empty cells so the columns after it still line up.
 * `rowspan` is NOT reconstructed — the cells below a spanned one shift left,
 * exactly as they do when the same table is pasted into a spreadsheet as
 * values. Saying so is better than a half-implementation that is right for
 * one merge and wrong for two.
 */
export function parseHtmlTable(html: string): Block | null {
  if (!html.trim()) return null;
  let document: Document;
  try {
    document = new DOMParser().parseFromString(html, "text/html");
  } catch {
    return null;
  }
  const table = document.querySelector("table");
  if (!table) return null;
  const block: Block = [];
  for (const row of Array.from(table.querySelectorAll("tr"))) {
    const cells: string[] = [];
    for (const cell of Array.from(row.querySelectorAll("th, td"))) {
      // `<br>` is a line break in a cell, not a space between two words.
      for (const br of Array.from(cell.querySelectorAll("br"))) {
        br.replaceWith("\n");
      }
      cells.push((cell.textContent ?? "").replace(/ /g, " ").trim());
      const span = Number((cell as HTMLTableCellElement).getAttribute("colspan") ?? "1");
      for (let extra = 1; extra < span; extra++) cells.push("");
    }
    if (cells.length > 0) block.push(cells);
  }
  return block.length > 0 ? block : null;
}

/**
 * Whatever the clipboard is holding, as a rectangle — or null if it holds
 * nothing this grid can use.
 *
 * A single cell of plain text is a perfectly good 1 × 1 rectangle: pasting a
 * word into a cell is the commonest paste there is, and it must not need a
 * table on the clipboard to work.
 */
export function parseClipboardTable(html: string | null, text: string | null): Block | null {
  const fromHtml = html ? parseHtmlTable(html) : null;
  if (fromHtml) return fromHtml;
  if (!text) return null;
  // No trailing-newline row: a spreadsheet ends its last line, and reading
  // that as an extra empty row would wipe the cells under a paste.
  const trimmed = text.replace(/\r?\n$/, "");
  if (trimmed === "") return null;
  const parsed = parseCsv(trimmed);
  return parsed.rows.length > 0 ? parsed.rows : null;
}
