<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/editor/tableEdit.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Cell formatting keeps numbers numeric](../../../../../contract.md)

Captured SHA-256: `0d73b0eb321c336ba3346f65b73a104a5067252326cb1ceb3a1740110218fdcf`.

<book:entry path="apps/web/src/editor/tableEdit.ts">
import type { RenderedSlot } from "./rendered";
import type { SlotContext } from "../elements/types";
import { structureOf } from "./wysiwyg";

/** Commit data and presentation together, so undo cannot separate them. */
export function replaceTable(slot: RenderedSlot, cx: SlotContext, csv: string, formats: string, agent = false): void {
  const view = cx.view;
  if (!view) return;
  const block = structureOf(view.state).blocks.find(b => b.from === slot.at && b.name === "table");
  if (!block) return;
  const opening = view.state.doc.sliceString(block.from, block.contentFrom);
  const stripped = opening.replace(/\s+formats=(?:"[^"]*"|'[^']*')/, "");
  const end = stripped.lastIndexOf(">");
  const next = formats ? `${stripped.slice(0, end)} formats="${formats}"${stripped.slice(end)}` : stripped;
  const prefix = view.state.doc.sliceString(block.contentFrom, block.contentTo).match(/^\r?\n/)?.[0] ?? "";
  const body = prefix + (csv.endsWith("\n") ? csv : `${csv}\n`);
  view.dispatch({ changes: [
    { from: block.from, to: block.contentFrom, insert: next },
    { from: block.contentFrom, to: block.contentTo, insert: body },
  ], userEvent: agent ? "input.uiAgentTable" : "input.table" });
}
</book:entry>

</book:doc>
