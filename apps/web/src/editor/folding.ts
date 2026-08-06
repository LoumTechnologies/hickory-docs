// Folding for the Document view, built on CodeMirror's NATIVE fold framework
// (@codemirror/language). Native folds are real fold ranges in CM's fold
// state, so the height map stays correct — this is deliberately NOT a
// hand-rolled hide-decoration scheme (see the layout rules in wysiwyg.ts).
//
// Two families of foldable regions, both computed from the cached hickDoc
// structure:
//  - Markdown headings: the heading line stays visible; everything up to the
//    next heading of the same or higher level folds away.
//  - Hick blocks (exec, file, copy, cut, when, container, session turns):
//    the opening tag line stays visible (with its chip/banner widget above
//    and, for exec, the cell panel below — widgets sit outside the folded
//    range), the body and closing tag fold away.

import { codeFolding, foldGutter, foldKeymap, foldService } from "@codemirror/language";
import type { EditorState, Extension } from "@codemirror/state";
import { keymap } from "@codemirror/view";
import { structureOf } from "./wysiwyg";
import type { HickDocStructure } from "./hickDoc";

export interface FoldRange {
  /** Fold start: end of the visible first line (heading / opening tag). */
  from: number;
  /** Fold end: end of the section / end of the closing tag. */
  to: number;
  kind: "heading" | "block";
  /** Block tag name, for block folds. */
  name?: string;
}

/** Hick blocks whose bodies fold. Session turns fold individually; `session`
 * itself folds as a whole conversation. */
export const FOLDABLE_BLOCKS = new Set([
  "exec",
  "file",
  "copy",
  "cut",
  "when",
  "container",
  "session",
  "user",
  "assistant",
  "observation",
]);

/**
 * All foldable ranges of a document, in order (outermost first at equal
 * starts). Pure — unit-tested against structures from parseHickDoc.
 */
export function computeFoldRanges(structure: HickDocStructure, text: string): FoldRange[] {
  const out: FoldRange[] = [];

  for (const b of structure.blocks) {
    if (!FOLDABLE_BLOCKS.has(b.name)) continue;
    // Self-closing / empty blocks have nothing to fold.
    if (b.contentTo <= b.contentFrom) continue;
    const lineEnd = text.indexOf("\n", b.from);
    // Single-line blocks (open + body + close on one line) aren't foldable.
    if (lineEnd < 0 || lineEnd >= b.to) continue;
    const to = Math.min(b.to, text.length);
    if (to > lineEnd) out.push({ from: lineEnd, to, kind: "block", name: b.name });
  }

  const hs = structure.headings;
  for (let i = 0; i < hs.length; i++) {
    const h = hs[i];
    // Section runs until the next heading of same-or-higher level.
    let end = text.length;
    for (let j = i + 1; j < hs.length; j++) {
      if (hs[j].level <= h.level) {
        end = hs[j].from - 1; // keep the newline that starts the next line
        break;
      }
    }
    // Trim trailing blank lines so the fold ends at real content.
    while (end > h.to && text[end - 1] === "\n") end--;
    if (end > h.to) out.push({ from: h.to, to: end, kind: "heading" });
  }

  out.sort((a, b) => a.from - b.from || b.to - a.to);
  return out;
}

/**
 * The fold range starting on the line [lineFrom, lineTo], or null. When
 * several ranges start on one line (nested blocks opening together), the
 * outermost wins. Pure — this is the foldService predicate.
 */
export function foldRangeForLine(
  ranges: FoldRange[],
  lineFrom: number,
  lineTo: number,
): { from: number; to: number } | null {
  let best: FoldRange | null = null;
  for (const r of ranges) {
    if (r.from < lineFrom) continue;
    if (r.from > lineTo) break; // ranges are sorted by from
    if (r.to > r.from && (!best || r.to > best.to)) best = r;
  }
  return best ? { from: best.from, to: best.to } : null;
}

// One fold-range computation per document version (same pattern as the
// structure cache in wysiwyg.ts).
const foldCache = new WeakMap<object, FoldRange[]>();

export function foldRangesOf(state: EditorState): FoldRange[] {
  const key = state.doc as unknown as object;
  let ranges = foldCache.get(key);
  if (!ranges) {
    ranges = computeFoldRanges(structureOf(state), state.doc.toString());
    foldCache.set(key, ranges);
  }
  return ranges;
}

/** The Document-view folding extension set: native fold state + gutter with
 * hickory-styled chevrons + the standard fold keymap. */
export function hickoryFolding(): Extension {
  return [
    codeFolding({
      placeholderDOM(_view, onclick) {
        const el = document.createElement("span");
        el.className = "cm-hick-fold-placeholder";
        el.textContent = "…";
        el.title = "Unfold";
        el.setAttribute("aria-label", "folded content — click to unfold");
        el.onclick = onclick;
        return el;
      },
    }),
    foldService.of((state, lineFrom, lineTo) =>
      foldRangeForLine(foldRangesOf(state), lineFrom, lineTo),
    ),
    foldGutter({
      markerDOM(open) {
        const el = document.createElement("span");
        el.className = `cm-fold-marker${open ? " cm-fold-open" : ""}`;
        el.textContent = "›"; // ›
        el.title = open ? "Fold" : "Unfold";
        return el;
      },
    }),
    keymap.of(foldKeymap),
  ];
}
