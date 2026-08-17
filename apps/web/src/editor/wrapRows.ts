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
