<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/editor/responseSend.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `9c9f19608b5cdcf04be57eca38c8ab211cc59d6f0c463f4f11345a1e64e1e484`.

<book:entry path="apps/web/src/editor/responseSend.ts">
import { StateEffect, StateField } from "@codemirror/state";
import { Decoration, EditorView, WidgetType } from "@codemirror/view";
import { responseStart } from "./protectedPrefix";

export const setSendState = StateEffect.define<{ enabled: boolean; label: string; visible: boolean }>();

class SendButton extends WidgetType {
  constructor(readonly enabled: boolean, readonly label: string, readonly send: () => void) { super(); }
  eq(other: SendButton) { return this.enabled === other.enabled && this.label === other.label; }
  toDOM() {
    const button = document.createElement("button");
    button.className = "btn btn-primary conversation-send";
    button.textContent = this.label;
    button.disabled = !this.enabled;
    button.addEventListener("mousedown", event => event.preventDefault());
    button.addEventListener("click", this.send);
    return button;
  }
  ignoreEvent() { return true; }
}

/** A view-only control after the draft; never inserted into session bytes. */
export function responseSend(send: () => void) {
  const status = StateField.define({
    create: () => ({ enabled: false, label: "Send", visible: true }),
    update(value, tr) {
      for (const effect of tr.effects) if (effect.is(setSendState)) value = effect.value;
      return value;
    },
  });
  return [status, EditorView.decorations.compute([status, responseStart, "doc"], state => {
    const { enabled, label, visible } = state.field(status);
    if (!visible) return Decoration.none;
    const hasDraft = !!state.doc.sliceString(state.field(responseStart)).trim();
    return Decoration.set([Decoration.widget({
      widget: new SendButton(enabled && hasDraft, label, send), side: 1, block: true,
    }).range(state.doc.length)]);
  })];
}
</book:entry>

</book:doc>
