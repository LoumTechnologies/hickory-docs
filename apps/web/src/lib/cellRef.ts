// A1 notation, in the browser.
//
// The twin of `CellRef` in crates/hick-formula/src/graph.rs, and deliberately
// only the half the UI needs: turning a position into a label, and back. The
// interesting work — which cells a formula depends on, what order they
// evaluate in, what a cycle is — lives on the host and stays there. Two
// implementations of THAT would eventually disagree, and a table that
// evaluated differently depending on who asked would be unusable.

/**
 * The letters of a zero-based column, e.g. 0 → "A", 26 → "AA".
 *
 * Bijective base-26, which is the one every spreadsheet uses and the one
 * `CellRef` on the host parses back. Drawn across the top of the grid, so the
 * reference a formula needs is the thing already on screen rather than
 * something the author counts out.
 */
export function columnLabel(column: number): string {
  let letters = "";
  let n = column + 1;
  while (n > 0) {
    const remainder = (n - 1) % 26;
    letters = String.fromCharCode(65 + remainder) + letters;
    n = Math.floor((n - 1) / 26);
  }
  return letters;
}

/** The A1 label of a zero-based position, e.g. (0, 0) → "A1". */
export function cellLabel(column: number, row: number): string {
  return `${columnLabel(column)}${row + 1}`;
}

/** Whether a cell's text is a formula: it begins with `=`, as in every
 * spreadsheet anybody has used. */
export function isFormula(text: string): boolean {
  return text.trimStart().startsWith("=");
}

/**
 * The position an A1 label names, e.g. "B3" → `{ column: 1, row: 2 }`.
 *
 * The inverse of `cellLabel`, and the only other half of A1 notation the UI
 * needs: the host answers in labels — a formula's value, a step's bindings —
 * and the grid has to find the cell those labels mean. Everything harder
 * than this (which cells a formula depends on, what order they evaluate in)
 * stays on the host, for the reason at the top of this file.
 *
 * Deliberately narrow where `CellRef::parse` is forgiving: this reads labels
 * this product produced, not labels a person pasted, so `$B$4` and lower
 * case are not its problem.
 */
export function parseCellLabel(label: string): { column: number; row: number } | null {
  const match = /^([A-Z]+)([0-9]+)$/.exec(label.trim());
  if (!match) return null;
  let column = 0;
  for (const letter of match[1]) {
    // Bijective base-26: there is no zero digit, which is why Z is followed
    // by AA.
    column = column * 26 + (letter.charCodeAt(0) - 64);
  }
  const row = Number(match[2]);
  if (row < 1) return null;
  return { column: column - 1, row: row - 1 };
}
