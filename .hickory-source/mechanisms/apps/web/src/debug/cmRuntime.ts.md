<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/cmRuntime.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-2517e693c335"></a>
## `apps/web/src/debug/cmRuntime.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Selected reading scope remains separate from complete edit state](../../../../../guarantees/agent/expanded-reading-projection/contract.md).

<a id="f-2517e693c335-setRuntimeReading"></a>
### `setRuntimeReading`

<storycode:copy id="f-2517e693c335-setRuntimeReading">
import { editorLanguageActions } from "../editor/languageActions";
import type { UiOutcome } from "../lib/uiActions";
import { profileRows } from "./profileRows";
import { RUNTIME_REVEAL, revealRuntimeEvent } from "./runtimeNavigation";
import { observationText, type RuntimeEvent } from "./runtimeObservations";
import { StateEffect, StateField } from "@codemirror/state";
import { Decoration, EditorView, WidgetType, type DecorationSet } from "@codemirror/view";
import type { ObservationReading } from "./observations";
import { selectObservations } from "./observations";
import { byteToChar } from "../lib/offsets";

export const setRuntimeReading = StateEffect.define<ObservationReading | null>();
</storycode:copy>
<a id="f-2517e693c335-disclosure"></a>
### `disclosure`

<storycode:copy id="f-2517e693c335-disclosure">
const disclosure = StateEffect.define<null>();
</storycode:copy>
<a id="f-2517e693c335-RuntimeWidget"></a>
### `RuntimeWidget`

<storycode:copy id="f-2517e693c335-RuntimeWidget">
class RuntimeWidget extends WidgetType {
  private owner = Symbol("inline-runtime");
  private removeListener = () => {};
  private view: EditorView | null = null;
  private details: HTMLDetailsElement | null = null;
  private selected: ObservationReading | null = null;
  constructor(readonly reading: ObservationReading, readonly location: string) { super(); }
  current() { return !!this.view?.dom.isConnected && inlineWidgets(this.view).includes(this); }
  isOpen() { return this.details?.open ?? false; }
  setOpen(open: boolean): UiOutcome {
    if (!this.current() || !this.details || !this.selected) return { status: "unavailable", reason: "Inline runtime source/widget retired" };
    const changed = this.details.open !== open;
    this.details.open = open;
    selectObservations(this.selected, open, false, this.owner);
    // Native toggle events also reach this handler; one state signal suffices.
    if (changed) this.view!.dispatch({ effects: disclosure.of(null) });
    return { status: "completed", value: { open, context_selection_requested: true } };
  }
  reveal(index: number): UiOutcome {
    const records = this.reading.annotations.filter(item => item.location === this.location);
    if (!this.current() || !this.isOpen() || !records[index]) return { status: "unavailable", reason: "Expand the current inline runtime annotation and select an available event" };
    revealRuntimeEvent(this.reading.scope.run, records[index].evidence);
    return { status: "completed", value: { navigation_requested: true, run: this.reading.scope.run, evidence: records[index].evidence } };
  }
  toDOM(view: EditorView) {
    this.view = view;
    const details = document.createElement("details"); details.className = "runtime-inline";
    const records = this.reading.annotations.filter(item => item.location === this.location);
    this.details = details;
    const summary = document.createElement("summary");
    summary.textContent = `${records.length} runtime observations · run ${this.reading.scope.run}`;
    const pre = document.createElement("pre"); pre.tabIndex = 0; pre.style.maxHeight = "12rem"; pre.style.overflow = "auto";
    pre.textContent = records.map(item => {
      const event = item.payload.event as RuntimeEvent;
      const body = observationText(event.observation);
      return `${item.evidence} · invocation ${item.scope.invocation ?? "unknown"}\n${body.replaceAll("\r", "\\r").replaceAll("\u001b", "\\x1b")}`;
    }).join("\n");
    const reading = { ...this.reading, annotations: records, locations: this.reading.locations.filter(item => item.id === this.location) };
    this.selected = reading;
    const select = () => {
      if (!this.current()) return;
      this.setOpen(details.open);
      view.dispatch({ effects: disclosure.of(null) });
    };
    details.addEventListener("toggle", select); details.append(summary, pre);
    const profiles = profileRows(records.map(item => item.payload.event as RuntimeEvent));
    if (profiles.length) {
      const table = document.createElement("table"); table.setAttribute("aria-label", "Inline function profile");
      const header = table.insertRow();
      for (const title of ["Function aggregate", "Total / primitive calls", "Self seconds", "Cumulative seconds (overlap)"]) { const th = document.createElement("th"); th.textContent = title; header.append(th); }
      for (const row of profiles) { const tr = table.insertRow(); for (const value of [row.function.name, `${row.function.total_calls} / ${row.function.primitive_calls}`, row.self ?? "unavailable", row.cumulative ?? "unavailable"]) tr.insertCell().textContent = String(value); }
      details.append(table);
    }
    details.dataset.run = reading.scope.run;
    details.dataset.events = records.map(item => item.evidence).join(" ");
    for (const [index, record] of records.entries()) {
      const button = document.createElement("button"); button.textContent = `Show ${record.evidence} in execution log`;
      button.addEventListener("click", () => this.reveal(index)); details.append(button);
    }
    const reveal = (event: Event) => {
      const detail = (event as CustomEvent<{ run: string; event: string }>).detail;
      if (this.current() && detail.run === reading.scope.run && records.some(item => item.evidence === detail.event)) {
        this.setOpen(true); details.scrollIntoView({ block: "nearest" }); summary.focus();
      }
    };
    window.addEventListener(RUNTIME_REVEAL, reveal); this.removeListener = () => window.removeEventListener(RUNTIME_REVEAL, reveal);
    return details;
  }
  destroy() { this.removeListener(); selectObservations(this.reading, false, false, this.owner); this.view = null; this.details = null; this.selected = null; }
  ignoreEvent() { return true; }
}
</storycode:copy>
<a id="f-2517e693c335-runtimeEditor"></a>
### `runtimeEditor`

<storycode:copy id="f-2517e693c335-runtimeEditor">
export const runtimeEditor = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, transaction) {
    if (transaction.docChanged) value = Decoration.none;
    for (const effect of transaction.effects) if (effect.is(setRuntimeReading)) {
      const reading = effect.value;
      if (!reading || new TextDecoder().decode(new Uint8Array(reading.source.bytes)) !== transaction.state.doc.toString()) { value = Decoration.none; continue; }
      const widgets = reading.locations.filter(location => reading.annotations.some(item => item.location === location.id)).map(location => {
        const at = byteToChar(transaction.state.doc.toString(), location.location.bytes?.start ?? 0);
        return Decoration.widget({ widget: new RuntimeWidget(reading, location.id), block: true, side: 1 }).range(transaction.state.doc.lineAt(at).to);
      });
      value = Decoration.set(widgets, true);
    }
    return value;
  },
  provide: field => [EditorView.decorations.from(field), inlineActions()],
});


</storycode:copy>
<a id="f-2517e693c335-inlineWidgets"></a>
### `inlineWidgets`

<storycode:copy id="f-2517e693c335-inlineWidgets">
function inlineWidgets(view: EditorView): RuntimeWidget[] {
  const widgets: RuntimeWidget[] = [];
  view.state.field(runtimeEditor, false)?.between(0, view.state.doc.length, (_from, _to, decoration) => {
    if (decoration.spec.widget instanceof RuntimeWidget) widgets.push(decoration.spec.widget);
  });
  return widgets;
}
</storycode:copy>
<a id="f-2517e693c335-inlineActions"></a>
### `inlineActions`

<storycode:copy id="f-2517e693c335-inlineActions">
function inlineActions() {
  const visible = (view: EditorView) => inlineWidgets(view).length > 0;
  const integer = { type: "number" as const, integer: true, minimum: 0, maximum: 100_000_000 };
  return [editorLanguageActions.of({ id: "inline-runtime:index", label: "Read inline runtime annotation page", visible, enabled: () => true,
    parameters: { first: integer, count: { ...integer, minimum: 1, maximum: 64 } },
    execute: async (view, args) => {
      const widgets = inlineWidgets(view), first = Number(args.first), count = Number(args.count);
      if (first > widgets.length) return { status: "unavailable", reason: "Index exceeds current inline annotations" };
      const value = { annotations: widgets.slice(first, first + count).map((widget, index) => ({ index: first + index, mounted: widget.current(), open: widget.isOpen(), events: widget.reading.annotations.filter(item => item.location === widget.location).length })), total: widgets.length, next: first + count < widgets.length ? first + count : null, content_read: false };
      return { status: "completed", value };
    } }), editorLanguageActions.of({ id: "inline-runtime:disclosure", label: "Set inline runtime annotation disclosure", visible, enabled: () => true,
    parameters: { index: integer, open: { type: "boolean" } },
    execute: async (view, args) => inlineWidgets(view)[Number(args.index)]?.setOpen(Boolean(args.open)) ?? { status: "unavailable", reason: "Inline annotation is unavailable" } }),
    editorLanguageActions.of({ id: "inline-runtime:read", label: "Read expanded inline runtime annotation bytes", visible: view => inlineWidgets(view).some(widget => widget.isOpen()), enabled: () => true,
      parameters: { index: integer, offset: integer, limit: { ...integer, minimum: 1, maximum: 8192 } },
      execute: async (view, args) => {
        const widget = inlineWidgets(view)[Number(args.index)];
        if (!widget?.current() || !widget.isOpen()) return { status: "unavailable", reason: "Expand the current inline annotation before reading" };
        const reading = widget.reading;
        const bytes = new TextEncoder().encode(JSON.stringify({ scope: reading.scope, source_revision: reading.source.revision, location: reading.locations.find(item => item.id === widget.location), annotations: reading.annotations.filter(item => item.location === widget.location), unsupported: reading.unsupported, historical_capture: true }));
        const offset = Number(args.offset), end = Math.min(bytes.length, offset + Number(args.limit));
        if (offset > bytes.length) return { status: "unavailable", reason: "Offset exceeds inline reading" };
        return { status: "completed", value: { bytes: [...bytes.slice(offset, end)], encoding: "JSON UTF-8", span: [offset, end], next: end < bytes.length ? end : null, total_bytes: bytes.length } };
      } }), editorLanguageActions.of({ id: "inline-runtime:reveal", label: "Show inline runtime event in execution log", visible: view => inlineWidgets(view).some(widget => widget.isOpen()), enabled: () => true,
        parameters: { index: integer, event: integer }, execute: async (view, args) => inlineWidgets(view)[Number(args.index)]?.reveal(Number(args.event)) ?? { status: "unavailable", reason: "Inline annotation is unavailable" } }),
  ];
}
</storycode:copy>

### Assemble `apps/web/src/debug/cmRuntime.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/cmRuntime.ts"><storycode:paste select="#f-2517e693c335-setRuntimeReading"/><storycode:paste select="#f-2517e693c335-disclosure"/><storycode:paste select="#f-2517e693c335-RuntimeWidget"/><storycode:paste select="#f-2517e693c335-runtimeEditor"/><storycode:paste select="#f-2517e693c335-inlineWidgets"/><storycode:paste select="#f-2517e693c335-inlineActions"/></storycode:file>

</storycode:doc>
