import { RangeSetBuilder, StateEffect, StateField, type Extension } from "@codemirror/state";
import { Decoration, EditorView, GutterMarker, ViewPlugin, gutter, type DecorationSet, type ViewUpdate } from "@codemirror/view";
import { onCoverageSelection, selectedCoverage, coveredLocations } from "./selection";
import { sha256 } from "../lib/reviews";

const setCoverage = StateEffect.define<{ line: number; count: number; label: string }[]>();
const rows = StateField.define<{ line: number; count: number; label: string }[]>({
  create: () => [], update(value, tr) { if (tr.docChanged) value = []; for (const e of tr.effects) if (e.is(setCoverage)) value = e.value; return value; },
});
const decorations = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, tr) {
    if (tr.docChanged || tr.effects.some(e => e.is(setCoverage))) {
      const ranges = new RangeSetBuilder<Decoration>();
      for (const row of tr.state.field(rows)) if (row.line <= tr.state.doc.lines && row.line > 0) {
        const at = tr.state.doc.line(row.line).from;
        ranges.add(at, at, Decoration.line({ class: row.count > 0 ? "cm-coverage-hit" : "cm-coverage-miss", attributes: { title: row.label } }));
      }
      value = ranges.finish();
    }
    return value;
  }, provide: field => EditorView.decorations.from(field),
});
class Marker extends GutterMarker {
  constructor(readonly count: number, readonly label: string) { super(); }
  eq(other: Marker) { return this.count === other.count && this.label === other.label; }
  toDOM() { const el = document.createElement("span"); el.textContent = this.count ? "●" : "○"; el.title = this.label; el.setAttribute("aria-label", this.label); return el; }
}
/** Source-checked decorations shared by plain, generated and literate editors. */
export function coverageGutter(path: () => string | undefined, serializedSource?: () => string | undefined): Extension {
  return [rows, decorations, ViewPlugin.fromClass(class {
    disposed = false; ticket = 0; off: () => void;
    constructor(readonly view: EditorView) { this.off = onCoverageSelection(() => { void this.refresh(); }); queueMicrotask(() => { void this.refresh(); }); }
    update(update: ViewUpdate) { if (update.docChanged) void this.refresh(); }
    async refresh() {
      const ticket = ++this.ticket, source = this.view.state.doc, run = selectedCoverage(), file = path();
      await Promise.resolve();
      let value: { line: number; count: number; label: string }[] = [];
      if (run && file && !run.source_changed_during_run && run.identity.source_hashes[file]) {
        const candidate = serializedSource?.();
        const text = candidate !== undefined && candidate.replace(/\r\n/g, "\n") === source.toString() ? candidate : source.toString();
        try {
          if (await sha256(text) === run.identity.source_hashes[file]) {
            // A literate line can contain multiple measured locations. A miss stays visible.
            const locations = new Map<number, { count: number; labels: string[] }>();
            for (const row of coveredLocations(run, file)) {
              const item = locations.get(row.line);
              const label = `${row.count > 0 ? "Covered" : "Uncovered executable"}: ${row.originalPath}:${row.originalLine}; count ${row.countsAvailable ? row.count : "unavailable"}; ${run.evidence?.attribution ?? "unavailable"} attribution; ${run.labels.join(", ")}; run ${run.id}; ${run.source_state}`;
              if (item) { item.count = Math.min(item.count, row.count); item.labels.push(label); } else locations.set(row.line, { count: row.count, labels: [label] });
            }
            value = [...locations].sort(([a], [b]) => a - b).map(([line, item]) => ({ line, count: item.count, label: item.labels.join("\n") }));
          }
        } catch { /* Without a source fingerprint, no current-source decoration. */ }
      }
      if (!this.disposed && ticket === this.ticket && this.view.state.doc === source) this.view.dispatch({ effects: setCoverage.of(value) });
    }
    destroy() { this.disposed = true; this.off(); }
  }), gutter({ class: "cm-coverage-gutter", markers(view) {
    const marks = new RangeSetBuilder<GutterMarker>();
    for (const row of view.state.field(rows)) if (row.line <= view.state.doc.lines && row.line > 0) { const at = view.state.doc.line(row.line).from; marks.add(at, at, new Marker(row.count, row.label)); }
    return marks.finish();
  } }), EditorView.theme({ ".cm-coverage-gutter": { color: "var(--muted)", fontSize: "0.7em" }, ".cm-coverage-hit": { backgroundColor: "color-mix(in srgb, var(--ok, green) 9%, transparent)" }, ".cm-coverage-miss": { backgroundColor: "color-mix(in srgb, var(--error, red) 13%, transparent)" } })];
}
