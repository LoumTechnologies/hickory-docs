<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/cmProvider.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c7d1b831a8aa"></a>
## `apps/web/src/debug/cmProvider.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-c7d1b831a8aa-setProviderReadings"></a>
### `setProviderReadings`

<storycode:copy id="f-c7d1b831a8aa-setProviderReadings">
import { StateEffect, StateField } from "@codemirror/state";
import { Decoration, EditorView, WidgetType, type DecorationSet } from "@codemirror/view";
import { createRoot, type Root } from "react-dom/client";
import { ProviderReading } from "./ProviderReading";
import type { ProviderReading as Reading } from "./providerObservations";
import { byteToChar } from "../lib/offsets";
export const setProviderReadings = StateEffect.define<Reading[]>();
</storycode:copy>
<a id="f-c7d1b831a8aa-ProviderWidget"></a>
### `ProviderWidget`

<storycode:copy id="f-c7d1b831a8aa-ProviderWidget">
class ProviderWidget extends WidgetType {
  private root?: Root;
  constructor(private reading: Reading) { super(); }
  toDOM() { const node = document.createElement("div"); node.className = "provider-inline"; this.root = createRoot(node); this.root.render(<ProviderReading reading={this.reading} />); return node; }
  destroy() { const root = this.root; queueMicrotask(() => root?.unmount()); }
  ignoreEvent() { return true; }
}
</storycode:copy>
<a id="f-c7d1b831a8aa-providerEditor"></a>
### `providerEditor`

<storycode:copy id="f-c7d1b831a8aa-providerEditor">
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
</storycode:copy>

### Assemble `apps/web/src/debug/cmProvider.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/cmProvider.tsx"><storycode:paste select="#f-c7d1b831a8aa-setProviderReadings"/><storycode:paste select="#f-c7d1b831a8aa-ProviderWidget"/><storycode:paste select="#f-c7d1b831a8aa-providerEditor"/></storycode:file>

</storycode:doc>
