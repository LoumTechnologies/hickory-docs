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
      widget: new SendButton(enabled && hasDraft, label, send), side: 1,
    }).range(state.doc.length)]);
  })];
}
