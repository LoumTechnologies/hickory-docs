// Pure geometry for the RIGHT line-number rail (editor/RightRail.tsx).
// CodeMirror's own gutter is left-only; the right rail mirrors it so a
// lineage ribbon lands beside a readable line number on BOTH edges of a
// pane. Everything here takes numbers and returns numbers — the DOM reading
// (viewportLineBlocks, rects) happens in the component, which keeps this
// unit-testable without a browser.

/** One viewport line block, as CodeMirror's height map reports it:
 * document-relative top, total painted height (wrapping included). */
export interface RailBlock {
  from: number;
  top: number;
  height: number;
}

/** A sub-block of a composite line block: the line's text, or a widget. */
export interface RailSubBlock {
  text: boolean;
  top: number;
  height: number;
}

/**
 * The extent the line NUMBER occupies: the text portion of a composite
 * block, not the whole block. A block widget (a file-block header, a cell
 * panel) merges into its line's height-map block, but the left gutter
 * aligns the number with the line's TEXT and leaves the widget's rows
 * blank — so the rail must too, or the two gutters put their gaps on
 * different sides of the same number (seen as "gap between 15 and 16 on
 * the left, between 16 and 17 on the right").
 */
export function textExtent(
  block: { top: number; height: number },
  children: readonly RailSubBlock[] | null,
): { top: number; height: number } {
  const text = children?.find((c) => c.text);
  return text ? { top: text.top, height: text.height } : block;
}

/** One rail entry: a line number and its rail-relative pixel box. */
export interface RailLine {
  line: number;
  top: number;
  height: number;
}

/**
 * The rail's entries for the current viewport: one per DOCUMENT line, not
 * per painted block. Consecutive blocks reporting the same line (a wrapped
 * line split across height-map entries) merge into one entry spanning both,
 * so the rail stays cell-for-cell with the left gutter.
 */
export function railLines(
  blocks: readonly RailBlock[],
  lineNumberAt: (pos: number) => number,
): RailLine[] {
  const out: RailLine[] = [];
  for (const block of blocks) {
    const line = lineNumberAt(block.from);
    const prev = out[out.length - 1];
    if (prev && prev.line === line) {
      prev.height = block.top + block.height - prev.top;
      continue;
    }
    out.push({ line, top: block.top, height: block.height });
  }
  return out;
}

/**
 * Rail width in `ch`, from the last line number it may need to show: the
 * digits plus breathing room, floored so a ten-line file still reads as a
 * rail rather than a hairline.
 */
export function railWidthCh(lastLine: number): number {
  return Math.max(2, String(Math.max(1, Math.floor(lastLine))).length) + 1.5;
}
