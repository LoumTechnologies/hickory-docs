// Display treatment for ordinary Markdown fences in a .md document.
//
// A fence is code even when Hickory has no special action for it. Keeping this
// separate from wysiwyg.ts makes that distinction explicit: cells and files
// are Hick structure; fences are Markdown prose with a code-shaped display.

import type { EditorState, Range } from "@codemirror/state";
import { Decoration } from "@codemirror/view";
import { highlightCode } from "./embedded";
import { proseCodeFences } from "./hickDoc";
import type { HickDocStructure } from "./hickDoc";

const line = Decoration.line({ class: "cm-fence-line" });
const first = Decoration.line({ class: "cm-fence-line cm-fence-first" });
const last = Decoration.line({ class: "cm-fence-line cm-fence-last" });
const only = Decoration.line({ class: "cm-fence-line cm-fence-first cm-fence-last" });
const marker = Decoration.mark({ class: "cm-fence-mark" });

/** Line and marker decorations that make every prose fence read as code. */
export function fenceLayoutRanges(
  state: EditorState,
  structure: HickDocStructure,
): Range<Decoration>[] {
  const ranges: Range<Decoration>[] = [];
  const doc = state.doc;
  for (const fence of proseCodeFences(structure, doc.toString())) {
    const firstLine = doc.lineAt(fence.from).number;
    const lastLine = doc.lineAt(Math.max(fence.from, fence.to - 1)).number;
    for (let n = firstLine; n <= lastLine; n++) {
      const deco = firstLine === lastLine ? only : n === firstLine ? first : n === lastLine ? last : line;
      ranges.push(deco.range(doc.line(n).from));
    }
    const opening = doc.lineAt(fence.from);
    ranges.push(marker.range(opening.from, opening.to));
    if (fence.closed) {
      const closing = doc.lineAt(fence.to);
      ranges.push(marker.range(closing.from, closing.to));
    }
  }
  return ranges;
}

/** Syntax-colour known fence languages; unknown tags still retain the frame. */
export function fenceHighlightRanges(
  state: EditorState,
  structure: HickDocStructure,
  visible: readonly { from: number; to: number }[],
  mark: (cls: string) => Decoration,
): Range<Decoration>[] {
  const ranges: Range<Decoration>[] = [];
  const doc = state.doc;
  for (const fence of proseCodeFences(structure, doc.toString())) {
    if (!visible.some(({ from, to }) => fence.to >= from && fence.from <= to)) continue;
    const language = fence.info.split(/\s+/, 1)[0];
    if (!language || fence.body.length === 0) continue;
    const bodyFrom = Math.min(doc.lineAt(fence.from).to + 1, doc.length);
    for (const span of highlightCode(fence.body, language)) {
      const from = bodyFrom + span.from;
      const to = bodyFrom + span.to;
      if (!visible.some((range) => to >= range.from && from <= range.to)) continue;
      ranges.push(mark(span.cls).range(from, to));
    }
  }
  return ranges;
}
