import { profileRows } from "./profileRows";
import { RUNTIME_REVEAL, revealRuntimeEvent } from "./runtimeNavigation";
import { observationText, type RuntimeEvent } from "./runtimeObservations";
import { StateEffect, StateField } from "@codemirror/state";
import { Decoration, EditorView, WidgetType, type DecorationSet } from "@codemirror/view";
import type { ObservationReading } from "./observations";
import { selectObservations } from "./observations";
import { byteToChar } from "../lib/offsets";

export const setRuntimeReading = StateEffect.define<ObservationReading | null>();
const expanded = new Set<string>();
class RuntimeWidget extends WidgetType {
  private owner = Symbol("inline-runtime");
  private removeListener = () => {};
  constructor(private reading: ObservationReading, private location: string) { super(); }
  toDOM() {
    const details = document.createElement("details"); details.className = "runtime-inline";
    const records = this.reading.annotations.filter(item => item.location === this.location);
    const key = `${this.reading.scope.run}:${this.location}`;
    const summary = document.createElement("summary");
    summary.textContent = `${records.length} runtime observations · run ${this.reading.scope.run}`;
    const pre = document.createElement("pre"); pre.tabIndex = 0; pre.style.maxHeight = "12rem"; pre.style.overflow = "auto";
    pre.textContent = records.map(item => {
      const event = item.payload.event as RuntimeEvent;
      const body = observationText(event.observation);
      return `${item.evidence} · invocation ${item.scope.invocation ?? "unknown"}\n${body.replaceAll("\r", "\\r").replaceAll("\u001b", "\\x1b")}`;
    }).join("\n");
    const reading = { ...this.reading, annotations: records, locations: this.reading.locations.filter(item => item.id === this.location) };
    details.open = expanded.has(key);
    const select = () => { if (details.open) expanded.add(key); else expanded.delete(key); selectObservations(reading, details.open, false, this.owner); };
    details.addEventListener("toggle", select); select(); details.append(summary, pre);
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
    for (const record of records) {
      const button = document.createElement("button"); button.textContent = `Show ${record.evidence} in execution log`;
      button.addEventListener("click", () => revealRuntimeEvent(reading.scope.run, record.evidence)); details.append(button);
    }
    const reveal = (event: Event) => {
      const detail = (event as CustomEvent<{ run: string; event: string }>).detail;
      if (detail.run === reading.scope.run && records.some(item => item.evidence === detail.event)) {
        details.open = true; details.scrollIntoView({ block: "nearest" }); summary.focus();
      }
    };
    window.addEventListener(RUNTIME_REVEAL, reveal); this.removeListener = () => window.removeEventListener(RUNTIME_REVEAL, reveal);
    return details;
  }
  destroy() { this.removeListener(); selectObservations(this.reading, false, false, this.owner); }
  ignoreEvent() { return true; }
}
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
  provide: field => EditorView.decorations.from(field),
});
