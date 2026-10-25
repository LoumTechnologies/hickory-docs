<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/formulaReferences.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A fill handle extends formulas](../../../../../contract.md)

Captured SHA-256: `85679d50d9c25d76a8a13f273f1fb64e821f87bb3c3a0ece102e60c75c729764`.

<book:entry path="apps/web/src/lib/formulaReferences.ts">
// The A1 references in a formula are part of the sheet, rather than part of
// Python or JavaScript. Keeping this rewrite here means structural edits move
// the data AND every formula's pointers to that data together.

import { cellLabel, parseCellLabel } from "./cellRef";

type Cell = { row: number; column: number };

/** Rewrite every A1 reference (including both ends of an A1 range). */
export function rewriteFormulaReferences(formula: string, move: (cell: Cell) => Cell): string {
  if (!formula.trimStart().startsWith("=")) return formula;
  return formula.replace(
    /(^|[^A-Za-z0-9_])((?:\$?[A-Za-z]{1,2}\$?\d+)(?::\$?[A-Za-z]{1,2}\$?\d+)?)(?![A-Za-z0-9_])/g,
    (_whole, before: string, reference: string) => {
      const rewritten = reference.replace(/\$?[A-Za-z]{1,2}\$?\d+/g, (part: string) => {
        const columnAbsolute = part.startsWith("$");
        const rowAbsolute = /\$\d+$/.test(part);
        const cell = parseCellLabel(part.replace(/\$/g, "").toUpperCase());
        if (!cell) return part;
        const moved = move(cell);
        const next = cellLabel(moved.column, moved.row);
        const match = /^([A-Z]+)(\d+)$/.exec(next)!;
        return `${columnAbsolute ? "$" : ""}${match[1]}${rowAbsolute ? "$" : ""}${match[2]}`;
      });
      return before + rewritten;
    },
  );
}

/** Shift references that moved with an inserted row or column. */
export function shiftFormulaReferences(
  formula: string,
  axis: "row" | "column",
  at: number,
): string {
  return rewriteFormulaReferences(formula, (cell) =>
    axis === "row"
      ? { ...cell, row: cell.row >= at ? cell.row + 1 : cell.row }
      : { ...cell, column: cell.column >= at ? cell.column + 1 : cell.column },
  );
}

/** Follow a rectangular cut-and-paste to its destination. */
export function moveFormulaReferences(
  formula: string,
  from: Cell,
  size: { rows: number; columns: number },
  to: Cell,
): string {
  return rewriteFormulaReferences(formula, (cell) => {
    const inside =
      cell.row >= from.row &&
      cell.row < from.row + size.rows &&
      cell.column >= from.column &&
      cell.column < from.column + size.columns;
    return inside
      ? { row: to.row + cell.row - from.row, column: to.column + cell.column - from.column }
      : cell;
  });
}

/** Copying changes relative references; insertion and cut/paste move even locks. */
export function copyFormulaReferences(formula: string, rows: number, columns: number, language = "javascript"): string {
  if (!formula.trimStart().startsWith("=")) return formula;
  // Skip string literals and comments. This is lexical, like reference discovery;
  // the expression still belongs to Python/JavaScript, not a spreadsheet parser.
  const strings = /"""[\s\S]*?"""|'''[\s\S]*?'''|"(?:\\[\s\S]|[^"\\])*"|'(?:\\[\s\S]|[^'\\])*'|`(?:\\[\s\S]|[^`\\])*`/.source;
  const references = /(?<![A-Za-z0-9_$])(\$?[A-Za-z]{1,2}\$?[1-9]\d*)(?![A-Za-z0-9_])/.source;
  const python = /^(python|python3|py)$/i.test(language.trim());
  const comments = python ? /#[^\n]*/.source : /\/\/[^\n]*|\/\*[\s\S]*?\*\//.source;
  const pattern = new RegExp(`${strings}|${comments}|${references}`, "g");
  return formula.replace(pattern,
    (whole: string, reference: string | undefined) => {
      if (!reference) return whole;
      const at = parseCellLabel(reference.replace(/\$/g, "").toUpperCase());
      if (!at) return whole;
      const columnAbsolute = reference.startsWith("$");
      const rowAbsolute = /\$\d+$/.test(reference);
      const row = at.row + (rowAbsolute ? 0 : rows);
      const column = at.column + (columnAbsolute ? 0 : columns);
      if (row < 0 || column < 0 || column >= 702) return "#REF!";
      const label = cellLabel(column, row).match(/^([A-Z]+)(\d+)$/)!;
      return `${columnAbsolute ? "$" : ""}${label[1]}${rowAbsolute ? "$" : ""}${label[2]}`;
    },
  );
}
</book:entry>

</book:doc>
