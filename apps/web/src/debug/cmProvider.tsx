import { StateEffect, StateField } from "@codemirror/state";
import { Decoration, EditorView, WidgetType, type DecorationSet } from "@codemirror/view";
import { createRoot, type Root } from "react-dom/client";
import { ProviderReading } from "./ProviderReading";
import type { ProviderReading as Reading } from "./providerObservations";
import { byteToChar } from "../lib/offsets";
export const setProviderReadings = StateEffect.define<Reading[]>();
class ProviderWidget extends WidgetType {
  private root?: Root;
  constructor(private reading: Reading) { super(); }
  toDOM() { const node = document.createElement("div"); node.className = "provider-inline"; this.root = createRoot(node); this.root.render(<ProviderReading reading={this.reading} />); return node; }
  destroy() { const root = this.root; queueMicrotask(() => root?.unmount()); }
  ignoreEvent() { return true; }
}
export const providerEditor = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, transaction) {
    if (transaction.docChanged) value = Decoration.none;
    for (const effect of transaction.effects) if (effect.is(setProviderReadings)) {
      const source = transaction.state.doc.toString();
      value = Decoration.set(effect.value.flatMap(reading => {
        if (!reading.source.bytes || new TextDecoder().decode(new Uint8Array(reading.source.bytes)) !== source) return [];
        return reading.annotations.flatMap(annotation => {
          if (!annotation.location?.location.bytes) return [];
          const at = byteToChar(source, annotation.location.location.bytes.start);
          return [Decoration.widget({ widget: new ProviderWidget({ ...reading, annotations: [annotation] }), block: true, side: 1 }).range(transaction.state.doc.lineAt(at).to)];
        });
      }), true);
    }
    return value;
  },
  provide: field => EditorView.decorations.from(field),
});
