import { StateEffect, StateField, type Extension } from "@codemirror/state";
import { Decoration, EditorView, ViewPlugin, WidgetType, type DecorationSet, type ViewUpdate } from "@codemirror/view";
import { editorLanguageActions } from "../editor/languageActions";
import { selectObservations, type ObservationReading } from "../debug/observations";
import { sha256 } from "../lib/reviews";
import { onPerformanceSelection, selectedPerformance, profileReading } from "./selection";
import type { ProfileRow } from "./types";

const setProfile = StateEffect.define<ObservationReading | null>();
const disclosure = StateEffect.define<null>();
const format = (n: number) => n.toLocaleString(undefined, { maximumSignificantDigits: 5 });
class Hotspot extends WidgetType {
  private owner = Symbol("profile-hotspot");
  private details: HTMLDetailsElement | null = null;
  constructor(readonly reading: ObservationReading) { super(); }
  open() { return this.details?.open ?? false; }
  disclose(open: boolean) { if (!this.details) return false; this.details.open = open; selectObservations(this.reading, open, this.reading.annotations.some(a => a.payload.source_state === "historical"), this.owner); return true; }
  toDOM(view: EditorView) {
    const details = document.createElement("details"); details.className = "runtime-inline performance-inline"; this.details = details;
    const summary = document.createElement("summary");
    const first = this.reading.annotations[0].payload, row = first.row as ProfileRow;
    summary.textContent = `◷ ${row.frame.name} · ${format(row.self_weight)} ${first.unit} self${this.reading.annotations.length > 1 ? ` · ${this.reading.annotations.length} functions` : ""}`;
    summary.title = "Function hotspot anchored here; this is not a measurement of this line";
    details.append(summary);
    const scope = document.createElement("p"); scope.textContent = `Run ${this.reading.scope.run} · ${first.source_state} · ${first.aggregation} · measured weight ${format(Number(first.measured_weight))} ${first.unit}. Inclusive costs overlap; samples are not calls.`; details.append(scope);
    const table = document.createElement("table"); table.setAttribute("aria-label", "Inline performance hotspots");
    const header = table.insertRow(); for (const title of ["Function", "Self", "Inclusive (overlap)", "Stack observations", "Total / primitive calls"]) { const th = document.createElement("th"); th.textContent = title; header.append(th); }
    for (const annotation of this.reading.annotations) { const payload = annotation.payload, row = payload.row as ProfileRow, tr = table.insertRow(); for (const value of [row.frame.name, `${format(row.self_weight)} ${payload.unit}`, `${format(row.inclusive_weight)} ${payload.unit}`, row.samples ?? "unavailable", `${row.total_calls ?? "unavailable"} / ${row.primitive_calls ?? "unavailable"}`]) tr.insertCell().textContent = String(value); }
    details.append(table);
    details.addEventListener("toggle", () => { if (this.details !== details || !view.dom.isConnected) return; this.disclose(details.open); view.dispatch({ effects: disclosure.of(null) }); });
    return details;
  }
  destroy() { selectObservations(this.reading, false, false, this.owner); this.details = null; }
  ignoreEvent() { return true; }
}
const hotspots = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, tr) {
    if (tr.docChanged) value = Decoration.none;
    for (const e of tr.effects) if (e.is(setProfile)) {
      const reading = e.value;
      value = reading ? Decoration.set(reading.locations.map(location => {
        const line = location.line + 1;
        if (line > tr.state.doc.lines) return null;
        const annotations = reading.annotations.filter(a => a.location === location.id);
        return Decoration.widget({ widget: new Hotspot({ ...reading, locations: [location], annotations }), block: true, side: 1 }).range(tr.state.doc.line(line).to);
      }).filter((range): range is NonNullable<typeof range> => range !== null), true) : Decoration.none;
    }
    return value;
  }, provide: field => EditorView.decorations.from(field),
});
function widgets(view: EditorView) { const list: Hotspot[] = []; view.state.field(hotspots).between(0, view.state.doc.length, (_a, _b, decoration) => { if (decoration.spec.widget instanceof Hotspot) list.push(decoration.spec.widget); }); return list; }
export function performanceInline(path: () => string | undefined, serializedSource?: () => string | undefined): Extension {
  const integer = { type: "number" as const, integer: true, minimum: 0, maximum: 100_000_000 };
  return [hotspots, ViewPlugin.fromClass(class {
    disposed = false; ticket = 0; off: () => void;
    constructor(readonly view: EditorView) { this.off = onPerformanceSelection(() => { void this.refresh(); }); queueMicrotask(() => { void this.refresh(); }); }
    update(update: ViewUpdate) { if (update.docChanged) void this.refresh(); }
    async refresh() {
      const ticket = ++this.ticket, source = this.view.state.doc, chosen = selectedPerformance(), file = path(); let reading: ObservationReading | null = null;
      await Promise.resolve();
      if (chosen && file && !chosen.run.source_changed) {
        const candidate = serializedSource?.(), serialized = candidate !== undefined && candidate.replace(/\r\n/g, "\n") === source.toString() ? candidate : source.toString();
        try { if (chosen.run.sources[file] === serialized && await sha256(serialized) === chosen.run.identity.source_hashes[file]) reading = profileReading(chosen.run, chosen.thread, file); } catch { /* No fingerprint means no current-source timing annotation. */ }
      }
      if (!this.disposed && ticket === this.ticket && this.view.state.doc === source) this.view.dispatch({ effects: setProfile.of(reading) });
    }
    destroy() { this.disposed = true; this.off(); }
  }), editorLanguageActions.of({ id: "performance-inline:read", label: "Read inline performance hotspot bytes", visible: view => widgets(view).some(w => w.open()), enabled: () => true, parameters: { index: integer, offset: integer, limit: { ...integer, minimum: 1, maximum: 8192 } }, execute: async (view, args) => {
    const widget = widgets(view)[Number(args.index)]; if (!widget?.open()) return { status: "unavailable", reason: "Expand an available performance hotspot first" };
    const bytes = new TextEncoder().encode(JSON.stringify(widget.reading)), offset = Number(args.offset), end = Math.min(bytes.length, offset + Number(args.limit));
    if (offset > bytes.length) return { status: "unavailable", reason: "Offset exceeds profile evidence" };
    return { status: "completed", value: { bytes: [...bytes.slice(offset, end)], encoding: "JSON UTF-8", total_bytes: bytes.length, next: end < bytes.length ? end : null } };
  } }), editorLanguageActions.of({ id: "performance-inline:disclose", label: "Expand or collapse inline performance hotspot", visible: view => widgets(view).length > 0, enabled: () => true, parameters: { index: integer, open: { type: "boolean" } }, execute: async (view, args) => {
    const widget = widgets(view)[Number(args.index)]; if (!widget?.disclose(Boolean(args.open))) return { status: "unavailable", reason: "Performance hotspot is unavailable" }; view.dispatch({ effects: disclosure.of(null) }); return { status: "completed", value: { open: widget.open(), context_selection_requested: true } };
  } })];
}
