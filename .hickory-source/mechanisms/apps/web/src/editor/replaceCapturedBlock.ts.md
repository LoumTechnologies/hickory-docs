<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/replaceCapturedBlock.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6ab01ff98ca7"></a>
## `apps/web/src/editor/replaceCapturedBlock.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-6ab01ff98ca7-replaceCapturedBlock"></a>
### `replaceCapturedBlock`

<storycode:copy id="f-6ab01ff98ca7-replaceCapturedBlock">
import type { EditorView } from "@codemirror/view";
import { structureOf } from "./wysiwyg";
import { renderableBlocks, renderBlock, type RenderedSlot } from "./rendered";

/** A rendered widget may only replace the exact editable block it captured. */
export function replaceCapturedBlock(view: EditorView | null, slot: Pick<RenderedSlot, "at" | "text">, text: string, userEvent: string): boolean {
  if (!view?.dom.isConnected || view.state.readOnly) return false;
  const block = renderableBlocks(structureOf(view.state), view.state.doc.toString()).find(block => block.from === slot.at);
  if (!block || view.state.doc.sliceString(block.contentFrom, block.contentTo) !== slot.text) return false;
  const body = block.name === "markdown-table" ? text : text.endsWith("\n") ? text : `${text}\n`;
  if (view.state.doc.sliceString(block.contentFrom, block.contentTo) === body) return true;
  const before = view.state.doc.toString(), expected = before.slice(0, block.contentFrom) + body + before.slice(block.contentTo);
  view.dispatch({ changes: { from: block.contentFrom, to: block.contentTo, insert: body },
    effects: block.name === "markdown-table" ? renderBlock.of(block.from) : [], userEvent });
  return view.state.doc.toString() === expected;
}
</storycode:copy>

### Assemble `apps/web/src/editor/replaceCapturedBlock.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/replaceCapturedBlock.ts"><storycode:paste select="#f-6ab01ff98ca7-replaceCapturedBlock"/></storycode:file>

</storycode:doc>
