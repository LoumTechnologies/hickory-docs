<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/wrapRows.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-72fa253bafd0"></a>
## `apps/web/src/editor/wrapRows.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-72fa253bafd0-RectSlice"></a>
### `RectSlice`

Visual-row measurement for soft-wrapped lines, shared by the left gutter's
wrap markers (editor/wrapGutter.ts) and the right rail (editor/RightRail.tsx).
Why measure the DOM at all: CodeMirror's height map reports a wrapped line
as ONE block whose height is the sum of its visual rows (verified live —
viewportLineBlocks never yields two blocks for one line), and it does not
say how many rows that height is. Dividing by `view.defaultLineHeight` is
wrong the moment a line is styled: a wrapped markdown heading measures two
35.5px rows against a 25.1px default (rounds to 3), and small-print styled
text measures 21.3px rows (four rows round to 3). The only truthful source
is the laid-out text itself: a Range over the line's element yields one
client rect per text fragment, and clustering those by vertical overlap
gives the exact rows — count AND boundaries.
The DOM read must happen in a read phase (requestMeasure / a layer's
`markers` callback), which is exactly where both callers live.

<storycode:copy id="f-72fa253bafd0-RectSlice">
// Visual-row measurement for soft-wrapped lines, shared by the left gutter's
// wrap markers (editor/wrapGutter.ts) and the right rail (editor/RightRail.tsx).
//
// Why measure the DOM at all: CodeMirror's height map reports a wrapped line
// as ONE block whose height is the sum of its visual rows (verified live —
// viewportLineBlocks never yields two blocks for one line), and it does not
// say how many rows that height is. Dividing by `view.defaultLineHeight` is
// wrong the moment a line is styled: a wrapped markdown heading measures two
// 35.5px rows against a 25.1px default (rounds to 3), and small-print styled
// text measures 21.3px rows (four rows round to 3). The only truthful source
// is the laid-out text itself: a Range over the line's element yields one
// client rect per text fragment, and clustering those by vertical overlap
// gives the exact rows — count AND boundaries.
//
// The DOM read must happen in a read phase (requestMeasure / a layer's
// `markers` callback), which is exactly where both callers live.

import type { EditorView } from "@codemirror/view";

/** A client-rect slice: what Range.getClientRects yields per text fragment. */
export interface RectSlice {
  top: number;
  bottom: number;
  width: number;
}

</storycode:copy>
<a id="f-72fa253bafd0-clusterRowTops"></a>
### `clusterRowTops`

The distinct visual rows in a set of client rects, as sorted row tops.
Rects arrive unsorted and finer-grained than rows: styled spans on one row
report separate rects whose tops differ by a few pixels (a `code` span sits
lower than the text around it). Two rects belong to one row when they
SUBSTANTIALLY overlap — at least half the shorter rect's height. Mere
touching is not enough: a wrapped heading's tall glyph boxes overlap their
next row's by a couple of pixels (measured live: bottom 170.6 against next
top 169.1), and an any-overlap rule would fuse two real rows into one.
Zero-width rects (collapsed fragments at wrap points) are noise.

<storycode:copy id="f-72fa253bafd0-clusterRowTops">
/**
 * The distinct visual rows in a set of client rects, as sorted row tops.
 *
 * Rects arrive unsorted and finer-grained than rows: styled spans on one row
 * report separate rects whose tops differ by a few pixels (a `code` span sits
 * lower than the text around it). Two rects belong to one row when they
 * SUBSTANTIALLY overlap — at least half the shorter rect's height. Mere
 * touching is not enough: a wrapped heading's tall glyph boxes overlap their
 * next row's by a couple of pixels (measured live: bottom 170.6 against next
 * top 169.1), and an any-overlap rule would fuse two real rows into one.
 * Zero-width rects (collapsed fragments at wrap points) are noise.
 */
export function clusterRowTops(rects: readonly RectSlice[]): number[] {
  const real = rects
    .filter((rect) => rect.width > 0 && rect.bottom > rect.top)
    .sort((a, b) => a.top - b.top);
  const rows: { top: number; bottom: number }[] = [];
  for (const rect of real) {
    const last = rows[rows.length - 1];
    const overlap = last ? last.bottom - rect.top : 0;
    const needed = last
      ? Math.min(rect.bottom - rect.top, last.bottom - last.top) / 2
      : Infinity;
    if (last && overlap >= needed) {
      last.bottom = Math.max(last.bottom, rect.bottom);
      continue;
    }
    rows.push({ top: rect.top, bottom: rect.bottom });
  }
  return rows.map((row) => row.top);
}

</storycode:copy>
<a id="f-72fa253bafd0-measureRowTops"></a>
### `measureRowTops`

The client-coordinate tops of the visual rows of the line at `pos`, or
null when the line has no measurable element (outside the viewport, or
fully replaced by a widget). One entry per row; a single-row line yields
one top. Call only from a measure/read phase.

<storycode:copy id="f-72fa253bafd0-measureRowTops">
/**
 * The client-coordinate tops of the visual rows of the line at `pos`, or
 * null when the line has no measurable element (outside the viewport, or
 * fully replaced by a widget). One entry per row; a single-row line yields
 * one top. Call only from a measure/read phase.
 */
export function measureRowTops(view: EditorView, pos: number): number[] | null {
  const at = view.domAtPos(pos);
  const node = at.node instanceof HTMLElement ? at.node : at.node.parentElement;
  const el = node?.closest?.(".cm-line") ?? null;
  if (!el) return null;
  const range = document.createRange();
  range.selectNodeContents(el);
  const tops = clusterRowTops(Array.from(range.getClientRects()));
  // An empty line has no text rects but still occupies one row.
  return tops.length > 0 ? tops : [el.getBoundingClientRect().top];
}
</storycode:copy>

### Assemble `apps/web/src/editor/wrapRows.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/wrapRows.ts"><storycode:paste select="#f-72fa253bafd0-RectSlice"/><storycode:paste select="#f-72fa253bafd0-clusterRowTops"/><storycode:paste select="#f-72fa253bafd0-measureRowTops"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-c65d95f8f1e2"></a>
## `apps/web/src/editor/wrapRows.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-c65d95f8f1e2-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-c65d95f8f1e2-expression_statement">
import { describe, expect, it } from "vitest";

import { clusterRowTops } from "./wrapRows";

describe("clusterRowTops", () => {
  it("returns one top per visual row for tidy row-per-rect input", () => {
    expect(
      clusterRowTops([
        { top: 0, bottom: 20, width: 100 },
        { top: 20, bottom: 40, width: 60 },
      ]),
    ).toEqual([0, 20]);
  });

  it("merges same-row fragments whose tops differ (styled spans), out of order", () => {
    // Real measured shape (a markdown line with inline-code spans): rects
    // arrive unsorted, and spans on one row report tops a few pixels apart
    // but overlapping. Tops 891 / 916 / 920 / 916 are TWO rows, not four.
    expect(
      clusterRowTops([
        { top: 891, bottom: 912, width: 300 },
        { top: 916, bottom: 937, width: 40 },
        { top: 920, bottom: 933, width: 55 },
        { top: 916, bottom: 937, width: 80 },
      ]),
    ).toEqual([891, 916]);
  });

  it("drops zero-width and zero-height rects (collapsed wrap-point fragments)", () => {
    expect(
      clusterRowTops([
        { top: 0, bottom: 20, width: 100 },
        { top: 20, bottom: 20, width: 100 },
        { top: 20, bottom: 40, width: 0 },
      ]),
    ).toEqual([0]);
  });

  it("is empty for no rects (an empty line)", () => {
    expect(clusterRowTops([])).toEqual([]);
  });

  it("keeps rows that merely graze each other distinct (a wrapped heading)", () => {
    // Real measured shape: a wrapped h1's tall glyph boxes overlap the next
    // row's by ~1.5px (bottom 170.6, next top 169.1). A couple of pixels is
    // not half a row — these are two rows, not one.
    expect(
      clusterRowTops([
        { top: 133.6, bottom: 170.6, width: 210 },
        { top: 133.6, bottom: 170.6, width: 25 },
        { top: 169.1, bottom: 206.6, width: 77 },
      ]),
    ).toEqual([133.6, 169.1]);
  });

  it("keeps three rows of small-print text distinct", () => {
    // Row height below the editor default (21.3px against 25.1px) — the
    // cluster boundary is overlap, not any assumed row height.
    expect(
      clusterRowTops([
        { top: 1157, bottom: 1174, width: 200 },
        { top: 1178, bottom: 1195, width: 210 },
        { top: 1199, bottom: 1216, width: 90 },
        { top: 1178, bottom: 1195, width: 30 },
        { top: 1199, bottom: 1216, width: 30 },
      ]),
    ).toEqual([1157, 1178, 1199]);
  });
});
</storycode:copy>

### Assemble `apps/web/src/editor/wrapRows.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/wrapRows.test.ts"><storycode:paste select="#f-c65d95f8f1e2-expression_statement"/></storycode:file>


</storycode:doc>
