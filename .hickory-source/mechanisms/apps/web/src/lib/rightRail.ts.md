<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/rightRail.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5467d907d8d2"></a>
## `apps/web/src/lib/rightRail.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-5467d907d8d2-RailBlock"></a>
### `RailBlock`

Pure geometry for the RIGHT line-number rail (editor/RightRail.tsx).
CodeMirror's own gutter is left-only; the right rail mirrors it so a
lineage ribbon lands beside a readable line number on BOTH edges of a
pane. Everything here takes numbers and returns numbers — the DOM reading
(viewportLineBlocks, rects) happens in the component, which keeps this
unit-testable without a browser.
One viewport line block, as CodeMirror's height map reports it:
document-relative top, total painted height (wrapping included), and —
when the component measured them — the document-relative tops of the
block's visual rows (a soft-wrapped line has several).

<storycode:copy id="f-5467d907d8d2-RailBlock">
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

</storycode:copy>
<a id="f-5467d907d8d2-RailSubBlock"></a>
### `RailSubBlock`

A sub-block of a composite line block: the line's text, or a widget.

<storycode:copy id="f-5467d907d8d2-RailSubBlock">
/** A sub-block of a composite line block: the line's text, or a widget. */
export interface RailSubBlock {
  text: boolean;
  top: number;
  height: number;
}

</storycode:copy>
<a id="f-5467d907d8d2-textExtent"></a>
### `textExtent`

The extent the line NUMBER occupies: the text portion of a composite
block, not the whole block. A block widget (a file-block header, a cell
panel) merges into its line's height-map block, but the left gutter
aligns the number with the line's TEXT and leaves the widget's rows
blank — so the rail must too, or the two gutters put their gaps on
different sides of the same number (seen as "gap between 15 and 16 on
the left, between 16 and 17 on the right").

<storycode:copy id="f-5467d907d8d2-textExtent">
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

</storycode:copy>
<a id="f-5467d907d8d2-rowBoxes"></a>
### `rowBoxes`

An extent divided at its measured visual-row tops: one box per row,
tiling the extent exactly. Each box runs from its row's top to the next
row's top (the last to the extent's bottom), so stacking the boxes
reproduces the extent to the pixel — glyph heights and inter-row leading
never enter into it. Tops outside the extent, out of order, or absent
leave the extent whole: one box.

<storycode:copy id="f-5467d907d8d2-rowBoxes">
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

</storycode:copy>
<a id="f-5467d907d8d2-RailLineKind"></a>
### `RailLineKind`

What a rail row shows: the line's number, or a wrap-continuation mark.

<storycode:copy id="f-5467d907d8d2-RailLineKind">
/** What a rail row shows: the line's number, or a wrap-continuation mark. */
export type RailLineKind = "number" | "wrap";

</storycode:copy>
<a id="f-5467d907d8d2-RailLine"></a>
### `RailLine`

One rail entry: a line number and its rail-relative pixel box.

<storycode:copy id="f-5467d907d8d2-RailLine">
/** One rail entry: a line number and its rail-relative pixel box. */
export interface RailLine {
  line: number;
  top: number;
  height: number;
  kind: RailLineKind;
}

</storycode:copy>
<a id="f-5467d907d8d2-railLines"></a>
### `railLines`

The rail's entries for the current viewport: one per VISUAL row, not per
document line. A soft-wrapped line's first row carries the number; every
continuation row is a "wrap" entry, so the tall cell reads as one logical
line continuing rather than unexplained blank space (and a brace spanning
it reads the same way). Two shapes of continuation are recognized:
measured row tops on a single block (what CodeMirror actually yields for
wrapping — one block per line), and, defensively, consecutive blocks
reporting the same line. Widget rows never reach here at all: callers
pass the TEXT extent (see textExtent), so a widget's height produces no
entry — that is what keeps this rail cell-for-cell with the left gutter.

<storycode:copy id="f-5467d907d8d2-railLines">
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

</storycode:copy>
<a id="f-5467d907d8d2-railWidthCh"></a>
### `railWidthCh`

Rail width in `ch`, from the last line number it may need to show: the
digits plus breathing room, floored so a ten-line file still reads as a
rail rather than a hairline.

<storycode:copy id="f-5467d907d8d2-railWidthCh">
/**
 * Rail width in `ch`, from the last line number it may need to show: the
 * digits plus breathing room, floored so a ten-line file still reads as a
 * rail rather than a hairline.
 */
export function railWidthCh(lastLine: number): number {
  return Math.max(2, String(Math.max(1, Math.floor(lastLine))).length) + 1.5;
}
</storycode:copy>

### Assemble `apps/web/src/lib/rightRail.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/rightRail.ts"><storycode:paste select="#f-5467d907d8d2-RailBlock"/><storycode:paste select="#f-5467d907d8d2-RailSubBlock"/><storycode:paste select="#f-5467d907d8d2-textExtent"/><storycode:paste select="#f-5467d907d8d2-rowBoxes"/><storycode:paste select="#f-5467d907d8d2-RailLineKind"/><storycode:paste select="#f-5467d907d8d2-RailLine"/><storycode:paste select="#f-5467d907d8d2-railLines"/><storycode:paste select="#f-5467d907d8d2-railWidthCh"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-6d983688fe4e"></a>
## `apps/web/src/lib/rightRail.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-6d983688fe4e-lineAt"></a>
### `lineAt`

<storycode:copy id="f-6d983688fe4e-lineAt">
import { describe, expect, it } from "vitest";

import { railLines, railWidthCh, rowBoxes, type RailBlock, textExtent } from "./rightRail";

// A ten-char-per-line "document": position 0..9 is line 1, 10..19 line 2, …
const lineAt = (pos: number) => Math.floor(pos / 10) + 1;

</storycode:copy>
<a id="f-6d983688fe4e-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-6d983688fe4e-expression_statement">
describe("railLines", () => {
  it("maps each unwrapped viewport block to a number entry", () => {
    const blocks: RailBlock[] = [
      { from: 0, top: 0, height: 20 },
      { from: 10, top: 20, height: 20 },
      { from: 20, top: 40, height: 20 },
    ];
    expect(railLines(blocks, lineAt)).toEqual([
      { line: 1, top: 0, height: 20, kind: "number" },
      { line: 2, top: 20, height: 20, kind: "number" },
      { line: 3, top: 40, height: 20, kind: "number" },
    ]);
  });

  it("splits a wrapped line into a number row plus wrap rows at its measured tops", () => {
    // Line 2 soft-wraps into three visual rows: ONE block (what
    // viewportLineBlocks actually yields for wrapping), with the rows'
    // measured tops. First row carries the number; the continuation rows are
    // wrap marks, and together they tile the block exactly.
    const blocks: RailBlock[] = [
      { from: 0, top: 0, height: 20 },
      { from: 10, top: 20, height: 60, rowTops: [20, 40, 60] },
      { from: 20, top: 80, height: 20 },
    ];
    expect(railLines(blocks, lineAt)).toEqual([
      { line: 1, top: 0, height: 20, kind: "number" },
      { line: 2, top: 20, height: 20, kind: "number" },
      { line: 2, top: 40, height: 20, kind: "wrap" },
      { line: 2, top: 60, height: 20, kind: "wrap" },
      { line: 3, top: 80, height: 20, kind: "number" },
    ]);
  });

  it("splits uneven rows (a styled line) where they were measured, not evenly", () => {
    // A wrapped heading: two rows of 35.5px against a 25px default line
    // height — the boundaries come from measurement, never from division.
    const blocks: RailBlock[] = [{ from: 0, top: 0, height: 71, rowTops: [0, 35.5] }];
    expect(railLines(blocks, lineAt)).toEqual([
      { line: 1, top: 0, height: 35.5, kind: "number" },
      { line: 1, top: 35.5, height: 35.5, kind: "wrap" },
    ]);
  });

  it("treats consecutive blocks of one line as continuations (defensive)", () => {
    // Should the height map ever split one line across blocks, the later
    // blocks are wrap rows — the number still renders exactly once.
    const blocks: RailBlock[] = [
      { from: 10, top: 0, height: 20 },
      { from: 15, top: 20, height: 20 },
    ];
    expect(railLines(blocks, lineAt)).toEqual([
      { line: 2, top: 0, height: 20, kind: "number" },
      { line: 2, top: 20, height: 20, kind: "wrap" },
    ]);
  });

  it("is empty for an empty viewport", () => {
    expect(railLines([], lineAt)).toEqual([]);
  });

  it("keeps a single unmeasured tall block whole: one number entry, no wrap rows", () => {
    // Without measured row tops there is nothing to split on — never guess.
    expect(railLines([{ from: 0, top: 5, height: 64 }], lineAt)).toEqual([
      { line: 1, top: 5, height: 64, kind: "number" },
    ]);
  });

  it("gives widget rows no entry while wrap rows of the same line get marks", () => {
    // The disambiguation the two kinds of "extra height" need: a widget row
    // was already cut away by textExtent (the block passed in IS the text
    // extent), so only genuine wrap rows inside that extent produce entries.
    const widgetAndText = textExtent({ top: 100, height: 90 }, [
      { text: false, top: 100, height: 40 }, // block widget above the line
      { text: true, top: 140, height: 50 }, // the line's text, wrapped twice
    ]);
    const blocks: RailBlock[] = [
      { from: 0, top: widgetAndText.top, height: widgetAndText.height, rowTops: [140, 165] },
    ];
    expect(railLines(blocks, lineAt)).toEqual([
      { line: 1, top: 140, height: 25, kind: "number" },
      { line: 1, top: 165, height: 25, kind: "wrap" },
    ]);
  });
});

</storycode:copy>
<a id="f-6d983688fe4e-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-6d983688fe4e-expression_statement-2">
describe("rowBoxes", () => {
  it("tiles the extent exactly at the given row tops", () => {
    expect(rowBoxes({ top: 10, height: 50 }, [10, 35])).toEqual([
      { top: 10, height: 25 },
      { top: 35, height: 25 },
    ]);
  });

  it("returns the whole extent for zero or one row top", () => {
    expect(rowBoxes({ top: 10, height: 50 }, [])).toEqual([{ top: 10, height: 50 }]);
    expect(rowBoxes({ top: 10, height: 50 }, [10])).toEqual([{ top: 10, height: 50 }]);
  });

  it("ignores tops outside the extent or out of order", () => {
    expect(rowBoxes({ top: 10, height: 40 }, [10, 5, 30, 30, 60])).toEqual([
      { top: 10, height: 20 },
      { top: 30, height: 20 },
    ]);
  });
});

</storycode:copy>
<a id="f-6d983688fe4e-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-6d983688fe4e-expression_statement-3">
describe("railWidthCh", () => {
  it("grows with the digit count", () => {
    expect(railWidthCh(9)).toBe(3.5);
    expect(railWidthCh(99)).toBe(3.5);
    expect(railWidthCh(100)).toBe(4.5);
    expect(railWidthCh(12345)).toBe(6.5);
  });

  it("never collapses below a readable floor", () => {
    expect(railWidthCh(1)).toBe(3.5);
    expect(railWidthCh(0)).toBe(3.5);
  });
});

</storycode:copy>
<a id="f-6d983688fe4e-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-6d983688fe4e-expression_statement-4">
describe("textExtent", () => {
  // Protects the gutter-parity behavior: the number sits beside the TEXT,
  // and a block widget's rows stay blank on BOTH gutters.
  it("uses the text sub-block of a composite line block", () => {
    const extent = textExtent({ top: 100, height: 60 }, [
      { text: false, top: 100, height: 40 }, // widget above the line
      { text: true, top: 140, height: 20 },
    ]);
    expect(extent).toEqual({ top: 140, height: 20 });
  });

  it("keeps the whole block when it is plain text", () => {
    expect(textExtent({ top: 10, height: 20 }, null)).toEqual({ top: 10, height: 20 });
  });

  it("falls back to the block when no text child exists", () => {
    const extent = textExtent({ top: 0, height: 30 }, [{ text: false, top: 0, height: 30 }]);
    expect(extent).toEqual({ top: 0, height: 30 });
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/rightRail.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/rightRail.test.ts"><storycode:paste select="#f-6d983688fe4e-lineAt"/><storycode:paste select="#f-6d983688fe4e-expression_statement"/><storycode:paste select="#f-6d983688fe4e-expression_statement-2"/><storycode:paste select="#f-6d983688fe4e-expression_statement-3"/><storycode:paste select="#f-6d983688fe4e-expression_statement-4"/></storycode:file>


</storycode:doc>
