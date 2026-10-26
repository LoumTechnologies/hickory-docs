<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/responseSend.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b22135a529bf"></a>
## `apps/web/src/editor/responseSend.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-b22135a529bf-setSendState"></a>
### `setSendState`

<storycode:copy id="f-b22135a529bf-setSendState">
import { StateEffect, StateField } from "@codemirror/state";
import { Decoration, EditorView, WidgetType } from "@codemirror/view";
import { responseStart } from "./protectedPrefix";

export const setSendState = StateEffect.define<{ enabled: boolean; label: string; visible: boolean }>();

</storycode:copy>
<a id="f-b22135a529bf-SendButton"></a>
### `SendButton`

<storycode:copy id="f-b22135a529bf-SendButton">
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

</storycode:copy>
<a id="f-b22135a529bf-responseSend"></a>
### `responseSend`

A view-only control after the draft; never inserted into session bytes.

<storycode:copy id="f-b22135a529bf-responseSend">
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
</storycode:copy>

### Assemble `apps/web/src/editor/responseSend.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/responseSend.ts"><storycode:paste select="#f-b22135a529bf-setSendState"/><storycode:paste select="#f-b22135a529bf-SendButton"/><storycode:paste select="#f-b22135a529bf-responseSend"/></storycode:file>

</storycode:doc>
