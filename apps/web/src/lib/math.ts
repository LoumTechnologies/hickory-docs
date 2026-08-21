// Where the LaTeX is in a piece of prose.
//
// Two notations, and they are genuinely different things rather than one
// thing with a size setting:
//
//  - `$$…$$` is DISPLAY maths: an equation that is a paragraph, set on its
//    own lines and centred. It may span lines.
//  - `$…$` is INLINE maths: a symbol inside a sentence, which must stay on
//    the sentence's line and share its baseline.
//
// The hard part is not the delimiters, it is the false positives. `$5 and $6`
// is money, not an equation, and a document about shell scripting is full of
// `$PATH` and `$1`. TeX's own rule is the one that works: an inline `$` opens
// maths only when the character after it is not a space, and closes only when
// the character before the closer is not a space. That alone throws out most
// prices — "$5 and $6" has "5 and 6" between two tight delimiters, so it
// needs the second rule too: a closing `$` may not be followed immediately by
// a digit, which is what "$6" looks like from the inside.
//
// Nothing here renders anything. This module is pure text-in / spans-out so
// the false-positive rules can be argued with in a test rather than in a
// browser.

/** One run of maths found in prose. */
export interface MathSpan {
  /** Whole span, both delimiters included. */
  from: number;
  to: number;
  /** Display maths (`$$`) sets on its own lines; inline shares the line. */
  display: boolean;
  /** The LaTeX between the delimiters, untrimmed. */
  source: string;
}

/** Escaped delimiters (`\$`) are dollars a reader wants to see. */
function escapedAt(text: string, index: number): boolean {
  let backslashes = 0;
  for (let i = index - 1; i >= 0 && text[i] === "\\"; i--) backslashes++;
  return backslashes % 2 === 1;
}

function inRanges(pos: number, ranges: readonly [number, number][]): boolean {
  return ranges.some(([from, to]) => pos >= from && pos < to);
}

/**
 * Every maths span in `text`, in document order, skipping anything inside
 * `verbatim` (fenced code, a cell's payload, a generated file's body).
 *
 * Display maths is matched first and greedily consumes its region, so the
 * `$` pair inside `$$ a $ b $$` is part of the display block rather than two
 * inline spans nested in it.
 */
export function mathSpans(
  text: string,
  verbatim: readonly [number, number][] = [],
): MathSpan[] {
  const spans: MathSpan[] = [];
  let i = 0;
  while (i < text.length) {
    if (text[i] !== "$" || escapedAt(text, i) || inRanges(i, verbatim)) {
      i++;
      continue;
    }
    if (text.startsWith("$$", i)) {
      const close = findClose(text, i + 2, "$$");
      if (close < 0) {
        i += 2;
        continue;
      }
      spans.push({
        from: i,
        to: close + 2,
        display: true,
        source: text.slice(i + 2, close),
      });
      i = close + 2;
      continue;
    }
    const close = findInlineClose(text, i + 1);
    if (close < 0) {
      i++;
      continue;
    }
    spans.push({ from: i, to: close + 1, display: false, source: text.slice(i + 1, close) });
    i = close + 1;
  }
  return spans;
}

function findClose(text: string, from: number, delimiter: string): number {
  for (let i = from; i <= text.length - delimiter.length; i++) {
    if (text.startsWith(delimiter, i) && !escapedAt(text, i)) return i;
  }
  return -1;
}

/**
 * The closing `$` of an inline span opened at `from`, or -1.
 *
 * The rules, all of them about not turning money and shell variables into
 * equations:
 *  - the opener must be followed by a non-space, so `$ 5` is never maths;
 *  - the closer must be preceded by a non-space, so `5 $` is never maths;
 *  - the closer may not be followed by a digit, which is what the SECOND
 *    price in "$5 and $6" looks like from inside the pair;
 *  - inline maths does not cross a blank line, and in practice does not
 *    cross a line at all — an unclosed `$` should give up at the end of its
 *    line rather than swallow the rest of the document.
 */
function findInlineClose(text: string, from: number): number {
  if (from >= text.length) return -1;
  const opener = text[from];
  if (opener === " " || opener === "\t" || opener === "\n") return -1;
  for (let i = from; i < text.length; i++) {
    const ch = text[i];
    if (ch === "\n") return -1;
    if (ch !== "$" || escapedAt(text, i)) continue;
    const before = text[i - 1];
    if (before === " " || before === "\t") return -1;
    const after = text[i + 1];
    if (after !== undefined && after >= "0" && after <= "9") return -1;
    return i;
  }
  return -1;
}
