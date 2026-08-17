// Pure geometry for the RIGHT line-number rail (editor/RightRail.tsx).
// CodeMirror's own gutter is left-only; the right rail mirrors it so a
// lineage ribbon lands beside a readable line number on BOTH edges of a
// pane. Everything here takes numbers and returns numbers — the DOM reading
// (viewportLineBlocks, rects) happens in the component, which keeps this
// unit-testable without a browser.

/** One viewport line block, as CodeMirror's height map reports it:
 * document-relative top, total painted height (wrapping included), and —
 * when the component measured them — the document-relative tops of the
 * block's visual rows (a soft-wrapped line has several). */
export interface RailBlock {
  from: number;
  top: number;
  height: number;
  rowTops?: readonly number[];
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

/**
 * An extent divided at its measured visual-row tops: one box per row,
 * tiling the extent exactly. Each box runs from its row's top to the next
 * row's top (the last to the extent's bottom), so stacking the boxes
 * reproduces the extent to the pixel — glyph heights and inter-row leading
 * never enter into it. Tops outside the extent, out of order, or absent
 * leave the extent whole: one box.
 */
export function rowBoxes(
  extent: { top: number; height: number },
  rowTops: readonly number[],
): { top: number; height: number }[] {
  const bottom = extent.top + extent.height;
  const cuts: number[] = [];
  for (const top of rowTops.slice(1)) {
    const prev = cuts[cuts.length - 1] ?? extent.top;
    if (top > prev && top < bottom) cuts.push(top);
  }
  const bounds = [extent.top, ...cuts, bottom];
  const out: { top: number; height: number }[] = [];
  for (let i = 0; i + 1 < bounds.length; i++) {
    out.push({ top: bounds[i], height: bounds[i + 1] - bounds[i] });
  }
  return out;
}

/** What a rail row shows: the line's number, or a wrap-continuation mark. */
export type RailLineKind = "number" | "wrap";

/** One rail entry: a line number and its rail-relative pixel box. */
export interface RailLine {
  line: number;
  top: number;
  height: number;
  kind: RailLineKind;
}

/**
 * The rail's entries for the current viewport: one per VISUAL row, not per
 * document line. A soft-wrapped line's first row carries the number; every
 * continuation row is a "wrap" entry, so the tall cell reads as one logical
 * line continuing rather than unexplained blank space (and a brace spanning
 * it reads the same way). Two shapes of continuation are recognized:
 * measured row tops on a single block (what CodeMirror actually yields for
 * wrapping — one block per line), and, defensively, consecutive blocks
 * reporting the same line. Widget rows never reach here at all: callers
 * pass the TEXT extent (see textExtent), so a widget's height produces no
 * entry — that is what keeps this rail cell-for-cell with the left gutter.
 */
export function railLines(
  blocks: readonly RailBlock[],
  lineNumberAt: (pos: number) => number,
): RailLine[] {
  const out: RailLine[] = [];
  for (const block of blocks) {
    const line = lineNumberAt(block.from);
    const prev = out[out.length - 1];
    const continues = prev !== undefined && prev.line === line;
    const boxes = rowBoxes(block, block.rowTops ?? []);
    boxes.forEach((box, i) => {
      const kind: RailLineKind = continues || i > 0 ? "wrap" : "number";
      out.push({ line, top: box.top, height: box.height, kind });
    });
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
