<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/editor/replaceCapturedBlock.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `b6a6eee877bde7512a0e345315f86461f3760fb40ec31a3d10a4ac94a7a9c3d3`.

<book:entry path="apps/web/src/editor/replaceCapturedBlock.ts">
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
</book:entry>

</book:doc>
