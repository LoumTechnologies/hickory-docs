// A1 notation, in the browser.
//
// The twin of `CellRef` in crates/hick-formula/src/graph.rs, and deliberately
// only the half the UI needs: turning a position into a label, and back. The
// interesting work — which cells a formula depends on, what order they
// evaluate in, what a cycle is — lives on the host and stays there. Two
// implementations of THAT would eventually disagree, and a table that
// evaluated differently depending on who asked would be unusable.

/** The A1 label of a zero-based position, e.g. (0, 0) → "A1". */
export function cellLabel(column: number, row: number): string {
  let letters = "";
  let n = column + 1;
  while (n > 0) {
    const remainder = (n - 1) % 26;
    letters = String.fromCharCode(65 + remainder) + letters;
    n = Math.floor((n - 1) / 26);
  }
  return `${letters}${row + 1}`;
}

/** Whether a cell's text is a formula: it begins with `=`, as in every
 * spreadsheet anybody has used. */
export function isFormula(text: string): boolean {
  return text.trimStart().startsWith("=");
}
