import { StateEffect, StateField } from "@codemirror/state";
import { Decoration, EditorView, WidgetType } from "@codemirror/view";
import { responseStart } from "./protectedPrefix";

export const setSendState = StateEffect.define<{ enabled: boolean; label: string; visible: boolean; allowEmpty?: boolean; reason?: string }>();

class SendButton extends WidgetType {
  constructor(readonly enabled: boolean, readonly label: string, readonly reason: string | undefined, readonly send: () => void) { super(); }
  eq(other: SendButton) { return this.enabled === other.enabled && this.label === other.label && this.reason === other.reason; }
  toDOM() {
    const button = document.createElement("button");
    button.className = "btn btn-primary conversation-send";
    button.textContent = this.label;
    button.title = this.reason ?? `${this.label} (${/Mac|iPhone|iPad/i.test(navigator.platform) ? "⌘" : "Ctrl"}+Enter)`;
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
    create: () => ({ enabled: false, label: "Send", visible: true, allowEmpty: false, reason: undefined as string | undefined }),
    update(value, tr) {
      for (const effect of tr.effects) if (effect.is(setSendState)) value = { ...effect.value, allowEmpty: effect.value.allowEmpty ?? false, reason: effect.value.reason };
      return value;
    },
  });
  return [status, EditorView.decorations.compute([status, responseStart, "doc"], state => {
    const { enabled, label, visible, allowEmpty, reason } = state.field(status);
    if (!visible) return Decoration.none;
    const hasDraft = !!state.doc.sliceString(state.field(responseStart)).trim();
    const canSend = enabled && (hasDraft || allowEmpty);
    const disabledReason = !enabled ? reason ?? "Wait for the agent to be ready." :
      !canSend ? "Enter a message or attach a file first." : undefined;
    return Decoration.set([Decoration.widget({
      widget: new SendButton(canSend, label, disabledReason, send), side: 1, block: true,
    }).range(state.doc.length)]);
  })];
}
