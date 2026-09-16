// Pointing at a cell while you are writing a formula.
//
// In every spreadsheet, `=` followed by a click on another cell writes that
// cell's reference into what you are typing. It is the feature that makes
// formulas usable without counting rows, and the reason people can write
// `=B2+B3` having never learned A1 notation.
//
// The whole difficulty is deciding whether a click MEANS a reference. While
// a formula is open, a click is either "put B2 here" or "I am done with this
// cell, take me to B2", and the grid has to guess right every time. A
// spreadsheet decides by where the caret is:
//
//   - Just after a reference this same act inserted → REPLACE it. Clicking
//     around to find the right cell should leave one reference, not five.
//   - Somewhere an operand could go — after `=`, `+`, `(`, `,`, a space →
//     INSERT. There is nothing there yet, so a reference is the only thing
//     the click could mean.
//   - Anywhere else, e.g. immediately after `42` or after a word → the
//     formula does not want an operand, so the click is a click: commit and
//     move on.
//
// This module is that decision, as a pure function over the text and the
// caret. It is deliberately not language-aware: the expression is in Python
// or JavaScript or Rust, and the panel cannot parse it (see hick-formula's
// lib.rs). The operand rule is a lexical one that holds in all of them.
//
// A click writes one cell; dragging writes an A1 range. The panel still only
// decides where text may go — the host expands a range and binds it as one
// list value before Python or JavaScript evaluates the expression.

/** Where the reference this act last inserted sits in the text, so the next
 * click replaces it rather than piling up beside it. */
export interface PointedAt {
  start: number;
  end: number;
}

export interface Pointing {
  /** The text with the reference in it. */
  text: string;
  /** Where to leave the caret: just after what was written. */
  caret: number;
  /** Where the reference landed, to be replaced by the next click. */
  pointed: PointedAt;
}

/**
 * The characters after which an operand could begin.
 *
 * Openers, separators, and operators — the shapes that are punctuation in
 * every language a formula might be written in. A letter, a digit, `)` or a
 * closing bracket are deliberately absent: after those the expression already
 * has its operand, and a click is a click.
 */
const OPERAND_AFTER = new Set([
  "=",
  "+",
  "-",
  "*",
  "/",
  "%",
  "^",
  "(",
  "[",
  "{",
  ",",
  ":",
  ";",
  "<",
  ">",
  "!",
  "&",
  "|",
  "~",
  "?",
  " ",
  "\t",
  "\n",
]);

/** Whether a reference could go in at `caret` — see the note above. */
export function acceptsReference(text: string, caret: number): boolean {
  const before = text.slice(0, Math.max(0, caret)).trimEnd();
  // Nothing before the caret at all is not an operand position in a formula:
  // a formula's first character is its `=`, and text with no `=` is not a
  // formula in the first place.
  if (before === "") return false;
  return OPERAND_AFTER.has(before[before.length - 1]);
}

/**
 * Write `label` into `text` at `caret`, or say the click did not mean that.
 *
 * `pointed` is where the previous click in this same edit put a reference;
 * when the caret is still sitting at the end of it, this one replaces it.
 * Answers null when the formula is not asking for an operand — the caller
 * then treats the click as an ordinary click, which is what it was.
 */
export function pointAt(
  text: string,
  caret: number,
  label: string,
  pointed?: PointedAt | null,
): Pointing | null {
  const at = Math.max(0, Math.min(caret, text.length));
  if (pointed && at === pointed.end && pointed.end > pointed.start) {
    const next = text.slice(0, pointed.start) + label + text.slice(pointed.end);
    return {
      text: next,
      caret: pointed.start + label.length,
      pointed: { start: pointed.start, end: pointed.start + label.length },
    };
  }
  if (!acceptsReference(text, at)) return null;
  const next = text.slice(0, at) + label + text.slice(at);
  return {
    text: next,
    caret: at + label.length,
    pointed: { start: at, end: at + label.length },
  };
}
