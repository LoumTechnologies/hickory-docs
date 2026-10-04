import { EditorState, StateField } from "@codemirror/state";
import { Decoration, EditorView, WidgetType, type DecorationSet } from "@codemirror/view";
import { structureOf } from "./wysiwyg";

class Bookkeeping extends WidgetType {
  constructor(readonly kind: string, readonly text: string, readonly from: number, readonly to: number) { super(); }
  eq(other: Bookkeeping) { return this.kind === other.kind && this.text === other.text && this.from === other.from && this.to === other.to; }
  toDOM() {
    const details = document.createElement("details");
    details.className = "conversation-bookkeeping";
    details.dataset.sessionFrom = String(this.from); details.dataset.sessionTo = String(this.to);
    details.appendChild(document.createElement("summary")).textContent = `Context · ${this.kind}`;
    details.appendChild(document.createElement("pre")).textContent = this.text;
    return details;
  }
  ignoreEvent() { return true; }
}
function decorations(state: EditorState): DecorationSet {
  const ranges = [];
  for (const block of structureOf(state).blocks) {
    if (block.name === "context") ranges.push(Decoration.replace({ block: true,
      widget: new Bookkeeping(block.attrs.kind ?? "record", state.doc.sliceString(block.contentFrom, block.contentTo), block.from, block.to),
    }).range(block.from, block.to));
  }
  return Decoration.set(ranges, true);
}
export const sessionBookkeeping = StateField.define<DecorationSet>({
  create: decorations,
  update: (value, tr) => tr.docChanged ? decorations(tr.state) : value,
  provide: field => EditorView.decorations.from(field),
});
