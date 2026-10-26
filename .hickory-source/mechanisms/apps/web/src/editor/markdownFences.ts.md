<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/markdownFences.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-94d646a3c2f6"></a>
## `apps/web/src/editor/markdownFences.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-94d646a3c2f6-line"></a>
### `line`

Display treatment for ordinary Markdown fences in a .md document.
A fence is code even when Hickory has no special action for it. Keeping this
separate from wysiwyg.ts makes that distinction explicit: cells and files
are Hick structure; fences are Markdown prose with a code-shaped display.

<storycode:copy id="f-94d646a3c2f6-line">
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
</storycode:copy>
<a id="f-94d646a3c2f6-first"></a>
### `first`

<storycode:copy id="f-94d646a3c2f6-first">
const first = Decoration.line({ class: "cm-fence-line cm-fence-first" });
</storycode:copy>
<a id="f-94d646a3c2f6-last"></a>
### `last`

<storycode:copy id="f-94d646a3c2f6-last">
const last = Decoration.line({ class: "cm-fence-line cm-fence-last" });
</storycode:copy>
<a id="f-94d646a3c2f6-only"></a>
### `only`

<storycode:copy id="f-94d646a3c2f6-only">
const only = Decoration.line({ class: "cm-fence-line cm-fence-first cm-fence-last" });
</storycode:copy>
<a id="f-94d646a3c2f6-marker"></a>
### `marker`

<storycode:copy id="f-94d646a3c2f6-marker">
const marker = Decoration.mark({ class: "cm-fence-mark" });

</storycode:copy>
<a id="f-94d646a3c2f6-fenceLayoutRanges"></a>
### `fenceLayoutRanges`

Line and marker decorations that make every prose fence read as code.

<storycode:copy id="f-94d646a3c2f6-fenceLayoutRanges">
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

</storycode:copy>
<a id="f-94d646a3c2f6-fenceHighlightRanges"></a>
### `fenceHighlightRanges`

Syntax-colour known fence languages; unknown tags still retain the frame.

<storycode:copy id="f-94d646a3c2f6-fenceHighlightRanges">
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
</storycode:copy>

### Assemble `apps/web/src/editor/markdownFences.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/markdownFences.ts"><storycode:paste select="#f-94d646a3c2f6-line"/><storycode:paste select="#f-94d646a3c2f6-first"/><storycode:paste select="#f-94d646a3c2f6-last"/><storycode:paste select="#f-94d646a3c2f6-only"/><storycode:paste select="#f-94d646a3c2f6-marker"/><storycode:paste select="#f-94d646a3c2f6-fenceLayoutRanges"/><storycode:paste select="#f-94d646a3c2f6-fenceHighlightRanges"/></storycode:file>

</storycode:doc>
