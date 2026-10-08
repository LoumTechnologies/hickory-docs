import type { RenderedSlot } from "./rendered";
import type { SlotContext } from "../elements/types";
import { structureOf } from "./wysiwyg";

/** Commit data and presentation together, so undo cannot separate them. */
export function replaceTable(slot: RenderedSlot, cx: SlotContext, csv: string, formats: string): void {
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
  ], userEvent: "input.table" });
}
