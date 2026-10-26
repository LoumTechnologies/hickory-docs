<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Profiles keep their measured scope

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a captured or imported performance profile, function costs retain the
collector's units, measured interval, per-thread/profile scope and attribution.
Sampling observations are not calls. Inclusive costs overlap and are never summed
as independent costs. A frame's source line anchors a function; it is not a
measurement of that line's execution time. Missing samples do not prove zero cost.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/profiles-keep-their-measured-scope.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-9f4934c1bafd"></a>
## `apps/web/src/performance/PerformanceTools.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Profiles keep their measured scope](contract.md).

<a id="f-9f4934c1bafd-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-9f4934c1bafd-expression_statement">
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { PerformancePanel } from "./PerformanceTools";
import { request } from "../api/client";
vi.mock("../api/client", () => ({ request: vi.fn() }));
</storycode:copy>
<a id="f-9f4934c1bafd-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-9f4934c1bafd-expression_statement-2">
vi.mock("../lib/revealLine", () => ({ openLocation: vi.fn() }));
</storycode:copy>
<a id="f-9f4934c1bafd-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-9f4934c1bafd-expression_statement-3">
afterEach(cleanup);
</storycode:copy>
<a id="f-9f4934c1bafd-run"></a>
### `run`

<storycode:copy id="f-9f4934c1bafd-run">
const run = { id: "1".repeat(32), mode: "manual-browser", status: "completed", source_state: "historical", source_changed: false, started_ms: 1, labels: [], sources: {}, mappings: {}, identity: { collector: "V8", version: "1", scope: ".", source_hashes: {} }, report: { limitations: ["sampling"], raw: "{}", format: "chrome-cpu", threads: [{ name: "main", unit: "seconds", measured_weight: 0.2, interval_start: 0, interval_end: 0.2, aggregation: "sampled", rows: [{ frame: { name: "hot", file: null, line: null }, self_weight: 0.1, inclusive_weight: 0.2, samples: 12, total_calls: null, primitive_calls: null }] }] }, output: "" };
</storycode:copy>
<a id="f-9f4934c1bafd-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-9f4934c1bafd-expression_statement-4">
// Guarantee: docs/guarantees/execution/profiles-keep-their-measured-scope.md
it("keeps sampled calls unavailable and unresolved native frames visible", async () => {
  vi.mocked(request).mockImplementation(async (_method, _path, body) => ((body as { action: string }).action === "list" ? { runs: [run] } : run) as never);
  render(<PerformancePanel close={() => {}} />);
  fireEvent.click(await screen.findByRole("button", { name: /manual-browser · completed/ }));
  expect(await screen.findByText("V8 · completed · historical source")).toBeTruthy();
  expect((screen.getByRole("button", { name: /hot · native\/unresolved/ }) as HTMLButtonElement).disabled).toBe(true);
  expect(screen.getByText("unavailable / unavailable")).toBeTruthy();
});
</storycode:copy>
<a id="f-9f4934c1bafd-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-9f4934c1bafd-expression_statement-5">
it("starts real manual profiling through the shared endpoint", async () => {
  vi.mocked(request).mockReset(); vi.mocked(request).mockImplementation(async (_method, _path, body) => ((body as { action: string }).action === "list" ? { runs: [] } : run) as never);
  render(<PerformancePanel close={() => {}} />);
  fireEvent.change(screen.getByLabelText("Page URL contains"), { target: { value: "localhost:3000" } });
  fireEvent.click(screen.getByRole("button", { name: "Start browser CPU profile" }));
  await waitFor(() => expect(request).toHaveBeenCalledWith("POST", "/api/performance", expect.objectContaining({ action: "browser-start", selector: "localhost:3000", url: "http://127.0.0.1:9222" })));
});
</storycode:copy>

### Assemble `apps/web/src/performance/PerformanceTools.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/performance/PerformanceTools.test.tsx"><storycode:paste select="#f-9f4934c1bafd-expression_statement"/><storycode:paste select="#f-9f4934c1bafd-expression_statement-2"/><storycode:paste select="#f-9f4934c1bafd-expression_statement-3"/><storycode:paste select="#f-9f4934c1bafd-run"/><storycode:paste select="#f-9f4934c1bafd-expression_statement-4"/><storycode:paste select="#f-9f4934c1bafd-expression_statement-5"/></storycode:file>

<a id="output-da78f5045853"></a>
## `apps/web/src/performance/inline.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Profiles keep their measured scope](contract.md).

<a id="f-da78f5045853-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-da78f5045853-expression_statement">
import { afterEach, expect, it, vi } from "vitest";
import { waitFor } from "@testing-library/react";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { performanceInline } from "./inline";
import { selectPerformance } from "./selection";
import { selectedObservations } from "../debug/observations";
import type { PerformanceRun } from "./types";
vi.mock("../lib/reviews", () => ({ sha256: async (text: string) => text }));
</storycode:copy>
<a id="f-da78f5045853-view"></a>
### `view`

<storycode:copy id="f-da78f5045853-view">
let view: EditorView | undefined;
</storycode:copy>
<a id="f-da78f5045853-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-da78f5045853-expression_statement-2">
afterEach(() => { view?.destroy(); view = undefined; selectPerformance(null); document.body.replaceChildren(); });
</storycode:copy>
<a id="f-da78f5045853-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-da78f5045853-expression_statement-3">
// Guarantee: docs/guarantees/execution/profiles-keep-their-measured-scope.md
it("expands measured hotspots into shared agent context and clears them after edits", async () => {
  const source = "# App\nfunction hot() {}\n";
  const run = { id: "sampled-run", source_changed: false, source_state: "current", sources: { "app.md": source }, labels: ["manual"], identity: { collector: "V8", version: "1", source_hashes: { "app.md": source } }, mappings: {}, report: { limitations: ["sampling"], threads: [{ name: "main", unit: "seconds", measured_weight: 0.1, aggregation: "sampled", interval_start: 0, interval_end: 0.1, rows: [{ frame: { name: "hot", file: "app.md", line: 2 }, self_weight: 0.02, inclusive_weight: 0.1, samples: 20, total_calls: null, primitive_calls: null }] }] } } as unknown as PerformanceRun;
  const host = document.createElement("div"); document.body.append(host);
  view = new EditorView({ parent: host, state: EditorState.create({ doc: source, extensions: [performanceInline(() => "app.md")] }) }); selectPerformance(run);
  await waitFor(() => expect(host.querySelector(".performance-inline summary")?.textContent).toContain("hot"));
  expect(selectedObservations("app.md", source)).toHaveLength(0);
  const details = host.querySelector("details")!; details.open = true; details.dispatchEvent(new Event("toggle"));
  expect(selectedObservations("app.md", source)[0].reading.annotations[0].payload).toMatchObject({ row: { total_calls: null, samples: 20 }, unit: "seconds" });
  expect(host.querySelector("table")?.textContent).toContain("unavailable / unavailable");
  view.dispatch({ changes: { from: 0, to: 1, insert: "changed" } });
  await waitFor(() => expect(host.querySelector(".performance-inline")).toBeNull());
  expect(selectedObservations("app.md", source)).toHaveLength(0);
});
</storycode:copy>

### Assemble `apps/web/src/performance/inline.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/performance/inline.test.ts"><storycode:paste select="#f-da78f5045853-expression_statement"/><storycode:paste select="#f-da78f5045853-view"/><storycode:paste select="#f-da78f5045853-expression_statement-2"/><storycode:paste select="#f-da78f5045853-expression_statement-3"/></storycode:file>

<a id="output-3ec4aa2fc9ed"></a>
## `apps/web/src/performance/inline.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Profiles keep their measured scope](contract.md).

<a id="f-3ec4aa2fc9ed-setProfile"></a>
### `setProfile`

<storycode:copy id="f-3ec4aa2fc9ed-setProfile">
import { StateEffect, StateField, type Extension } from "@codemirror/state";
import { Decoration, EditorView, ViewPlugin, WidgetType, type DecorationSet, type ViewUpdate } from "@codemirror/view";
import { editorLanguageActions } from "../editor/languageActions";
import { selectObservations, type ObservationReading } from "../debug/observations";
import { sha256 } from "../lib/reviews";
import { onPerformanceSelection, selectedPerformance, profileReading } from "./selection";
import type { ProfileRow } from "./types";

const setProfile = StateEffect.define<ObservationReading | null>();
</storycode:copy>
<a id="f-3ec4aa2fc9ed-disclosure"></a>
### `disclosure`

<storycode:copy id="f-3ec4aa2fc9ed-disclosure">
const disclosure = StateEffect.define<null>();
</storycode:copy>
<a id="f-3ec4aa2fc9ed-format"></a>
### `format`

<storycode:copy id="f-3ec4aa2fc9ed-format">
const format = (n: number) => n.toLocaleString(undefined, { maximumSignificantDigits: 5 });
</storycode:copy>
<a id="f-3ec4aa2fc9ed-Hotspot"></a>
### `Hotspot`

<storycode:copy id="f-3ec4aa2fc9ed-Hotspot">
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
</storycode:copy>
<a id="f-3ec4aa2fc9ed-hotspots"></a>
### `hotspots`

<storycode:copy id="f-3ec4aa2fc9ed-hotspots">
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
</storycode:copy>
<a id="f-3ec4aa2fc9ed-widgets"></a>
### `widgets`

<storycode:copy id="f-3ec4aa2fc9ed-widgets">
function widgets(view: EditorView) { const list: Hotspot[] = []; view.state.field(hotspots).between(0, view.state.doc.length, (_a, _b, decoration) => { if (decoration.spec.widget instanceof Hotspot) list.push(decoration.spec.widget); }); return list; }
</storycode:copy>
<a id="f-3ec4aa2fc9ed-performanceInline"></a>
### `performanceInline`

<storycode:copy id="f-3ec4aa2fc9ed-performanceInline">
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
</storycode:copy>

### Assemble `apps/web/src/performance/inline.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/performance/inline.ts"><storycode:paste select="#f-3ec4aa2fc9ed-setProfile"/><storycode:paste select="#f-3ec4aa2fc9ed-disclosure"/><storycode:paste select="#f-3ec4aa2fc9ed-format"/><storycode:paste select="#f-3ec4aa2fc9ed-Hotspot"/><storycode:paste select="#f-3ec4aa2fc9ed-hotspots"/><storycode:paste select="#f-3ec4aa2fc9ed-widgets"/><storycode:paste select="#f-3ec4aa2fc9ed-performanceInline"/></storycode:file>

<a id="output-22cf5dd7fa52"></a>
## `apps/web/src/performance/selection.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Profiles keep their measured scope](contract.md).

<a id="f-22cf5dd7fa52-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-22cf5dd7fa52-expression_statement">
import { expect, it } from "vitest";
import { profileReading } from "./selection";
import type { PerformanceRun } from "./types";
// Guarantee: docs/guarantees/execution/profiles-keep-their-measured-scope.md
it("anchors function aggregates in literate source and retains scope, units and unknown calls", () => {
  const source = "# App\nfunction hot() {}\n";
  const run = { id: "profile", source_state: "current", sources: { "app.md": source }, identity: { collector: "V8", version: "1", source_hashes: { "app.md": "hash" } }, labels: ["manual"], mappings: { "app.ts": { 1: { path: "app.md", line: 2 } } }, report: { limitations: ["sampling"], threads: [{ name: "main", unit: "seconds", aggregation: "sampled", measured_weight: 0.1, interval_start: 0, interval_end: 0.1, rows: [{ frame: { name: "hot", file: "app.ts", line: 1 }, self_weight: 0.02, inclusive_weight: 0.1, samples: 20, total_calls: null, primitive_calls: null }] }] } } as unknown as PerformanceRun;
  const reading = profileReading(run, 0, "app.md")!;
  expect(reading.locations[0].line).toBe(1);
  expect(reading.annotations[0].payload).toMatchObject({ kind: "profile", unit: "seconds", row: { samples: 20, total_calls: null }, labels: ["manual"] });
  expect(new TextDecoder().decode(new Uint8Array(reading.source.bytes))).toBe(source);
  expect(profileReading(run, 0, "other.md")).toBeNull();
});
</storycode:copy>

### Assemble `apps/web/src/performance/selection.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/performance/selection.test.ts"><storycode:paste select="#f-22cf5dd7fa52-expression_statement"/></storycode:file>

<a id="output-0c82852f57d0"></a>
## `crates/hick-profile/tests/metrics.rs`

Related contracts: [Profiles keep their measured scope](contract.md).

<a id="f-0c82852f57d0-sampled_weights_preserve_threads_and_do_not_invent_calls"></a>
### `sampled_weights_preserve_threads_and_do_not_invent_calls`

<storycode:copy id="f-0c82852f57d0-sampled_weights_preserve_threads_and_do_not_invent_calls">
// Guarantee: docs/guarantees/execution/profiles-keep-their-measured-scope.md
use hick_profile::parse;
use serde_json::json;
#[test]
fn sampled_weights_preserve_threads_and_do_not_invent_calls() {
    let raw=json!({"shared":{"frames":[{"name":"outer","file":"app.rs","line":1},{"name":"inner","file":"app.rs","line":2}]},"profiles":[{"type":"sampled","name":"main","unit":"milliseconds","startValue":0,"endValue":5,"samples":[[0,1],[0]],"weights":[3,2]},{"type":"sampled","name":"worker","unit":"milliseconds","startValue":0,"endValue":2,"samples":[[1]],"weights":[2]}]}).to_string();
    let r = parse("speedscope", &raw).unwrap();
    assert_eq!(r.threads.len(), 2);
    let main = &r.threads[0];
    assert_eq!(main.unit, "seconds");
    assert_eq!(main.measured_weight, 0.005);
    let outer = main.rows.iter().find(|r| r.frame.name == "outer").unwrap();
    assert_eq!(outer.self_weight, 0.002);
    assert_eq!(outer.inclusive_weight, 0.005);
    assert_eq!(outer.total_calls, None);
    assert_eq!(outer.samples, Some(2));
    assert_eq!(r.raw, raw);
}
</storycode:copy>
<a id="f-0c82852f57d0-recursion_does_not_multiply_inclusive_stack_weight"></a>
### `recursion_does_not_multiply_inclusive_stack_weight`

<storycode:copy id="f-0c82852f57d0-recursion_does_not_multiply_inclusive_stack_weight">
#[test]
fn recursion_does_not_multiply_inclusive_stack_weight() {
    let raw=json!({"shared":{"frames":[{"name":"recursive"}]},"profiles":[{"type":"sampled","unit":"seconds","startValue":0,"endValue":1,"samples":[[0,0,0]],"weights":[1]}]}).to_string();
    let r = parse("speedscope", &raw).unwrap();
    let row = &r.threads[0].rows[0];
    assert_eq!(row.self_weight, 1.);
    assert_eq!(row.inclusive_weight, 1.);
    assert_eq!(row.samples, Some(1));
}
</storycode:copy>
<a id="f-0c82852f57d0-evented_profiles_keep_self_and_overlapping_inclusive_costs_separate"></a>
### `evented_profiles_keep_self_and_overlapping_inclusive_costs_separate`

<storycode:copy id="f-0c82852f57d0-evented_profiles_keep_self_and_overlapping_inclusive_costs_separate">
#[test]
fn evented_profiles_keep_self_and_overlapping_inclusive_costs_separate() {
    let raw=json!({"shared":{"frames":[{"name":"outer"},{"name":"inner"}]},"profiles":[{"type":"evented","unit":"seconds","startValue":0,"endValue":5,"events":[{"type":"O","at":0,"frame":0},{"type":"O","at":1,"frame":1},{"type":"C","at":4,"frame":1},{"type":"C","at":5,"frame":0}]}]}).to_string();
    let r = parse("speedscope", &raw).unwrap();
    let t = &r.threads[0];
    assert_eq!(t.measured_weight, 5.);
    let outer = t.rows.iter().find(|r| r.frame.name == "outer").unwrap();
    assert_eq!(outer.self_weight, 2.);
    assert_eq!(outer.inclusive_weight, 5.);
    assert_eq!(outer.samples, None);
}
</storycode:copy>
<a id="f-0c82852f57d0-v8_uses_microsecond_deltas_and_zero_based_locations"></a>
### `v8_uses_microsecond_deltas_and_zero_based_locations`

<storycode:copy id="f-0c82852f57d0-v8_uses_microsecond_deltas_and_zero_based_locations">
#[test]
fn v8_uses_microsecond_deltas_and_zero_based_locations() {
    let raw=json!({"nodes":[{"id":1,"callFrame":{"functionName":"(root)","url":"","lineNumber":-1},"children":[2]},{"id":2,"callFrame":{"functionName":"hot","url":"file:///app.ts","lineNumber":4,"columnNumber":2,"scriptId":"1"}}],"startTime":100,"endTime":3100,"samples":[2,2],"timeDeltas":[1000,2000]}).to_string();
    let r = parse("chrome-cpu", &raw).unwrap();
    let hot = r.threads[0]
        .rows
        .iter()
        .find(|r| r.frame.name == "hot")
        .unwrap();
    assert_eq!(hot.frame.line, Some(5));
    assert_eq!(hot.frame.column, Some(3));
    assert_eq!(hot.self_weight, 0.003);
    assert_eq!(hot.total_calls, None);
}
</storycode:copy>
<a id="f-0c82852f57d0-missing_v8_deltas_remain_samples_not_estimated_time"></a>
### `missing_v8_deltas_remain_samples_not_estimated_time`

<storycode:copy id="f-0c82852f57d0-missing_v8_deltas_remain_samples_not_estimated_time">
#[test]
fn missing_v8_deltas_remain_samples_not_estimated_time() {
    let r=parse("chrome-cpu",r#"{"nodes":[{"id":1,"callFrame":{"functionName":"hot"}}],"startTime":0,"endTime":100,"samples":[1]}"#).unwrap();
    assert_eq!(r.threads[0].unit, "samples");
    assert_eq!(r.threads[0].rows[0].self_weight, 1.);
}
</storycode:copy>
<a id="f-0c82852f57d0-cprofile_retains_recursion_counts_and_inclusive_times"></a>
### `cprofile_retains_recursion_counts_and_inclusive_times`

<storycode:copy id="f-0c82852f57d0-cprofile_retains_recursion_counts_and_inclusive_times">
#[test]
fn cprofile_retains_recursion_counts_and_inclusive_times() {
    let raw=json!({"elapsed_seconds":2,"functions":[{"name":"recursive","file":"app.py","line":3,"total_calls":9,"primitive_calls":1,"self_seconds":0.2,"cumulative_seconds":1.8}]}).to_string();
    let r = parse("cprofile-json", &raw).unwrap();
    let row = &r.threads[0].rows[0];
    assert_eq!(row.total_calls, Some(9));
    assert_eq!(row.primitive_calls, Some(1));
    assert_eq!(row.samples, None);
    assert_eq!(row.inclusive_weight, 1.8);
}
</storycode:copy>
<a id="f-0c82852f57d0-malformed_or_unknown_profiles_never_become_zero_cost"></a>
### `malformed_or_unknown_profiles_never_become_zero_cost`

<storycode:copy id="f-0c82852f57d0-malformed_or_unknown_profiles_never_become_zero_cost">
#[test]
fn malformed_or_unknown_profiles_never_become_zero_cost() {
    for (format, raw) in [
        ("other", "{}"),
        (
            "chrome-cpu",
            r#"{"nodes":[],"startTime":0,"endTime":1,"samples":[2],"timeDeltas":[1]}"#,
        ),
        (
            "speedscope",
            r#"{"shared":{"frames":[]},"profiles":[{"type":"sampled","unit":"seconds","startValue":0,"endValue":1,"samples":[[]]}]}"#,
        ),
        (
            "speedscope",
            r#"{"shared":{"frames":[{"name":"a"}]},"profiles":[{"type":"evented","unit":"seconds","startValue":0,"endValue":1,"events":[{"type":"C","frame":0,"at":0}]}]}"#,
        ),
    ] {
        assert!(parse(format, raw).is_err());
    }
}
</storycode:copy>
<a id="f-0c82852f57d0-processed_firefox_preserves_symbolicated_rust_locations_and_sample_units"></a>
### `processed_firefox_preserves_symbolicated_rust_locations_and_sample_units`

<storycode:copy id="f-0c82852f57d0-processed_firefox_preserves_symbolicated_rust_locations_and_sample_units">
#[test]
fn processed_firefox_preserves_symbolicated_rust_locations_and_sample_units() {
    let raw=json!({"threads":[{"name":"main","tid":7,"stringArray":["crate::hot","src/main.rs"],"funcTable":{"name":[0],"fileName":[1],"lineNumber":[12],"columnNumber":[1]},"frameTable":{"func":[0]},"stackTable":{"frame":[0],"prefix":[null]},"samples":{"stack":[0,0],"time":[1,3],"weightType":"samples","weight":[1,2]}}]}).to_string();
    let r = parse("firefox", &raw).unwrap();
    let t = &r.threads[0];
    assert_eq!(t.unit, "samples");
    assert_eq!(t.measured_weight, 3.);
    assert_eq!(t.rows[0].frame.file.as_deref(), Some("src/main.rs"));
    assert_eq!(t.rows[0].frame.line, Some(12));
    assert_eq!(t.rows[0].total_calls, None);
}
</storycode:copy>
<a id="f-0c82852f57d0-unconverted_firefox_profiles_are_unavailable_instead_of_zero"></a>
### `unconverted_firefox_profiles_are_unavailable_instead_of_zero`

<storycode:copy id="f-0c82852f57d0-unconverted_firefox_profiles_are_unavailable_instead_of_zero">
#[test]
fn unconverted_firefox_profiles_are_unavailable_instead_of_zero() {
    assert!(parse("firefox", r#"{"threads":[{"samples":{}}]}"#).is_err());
}
</storycode:copy>

### Assemble `crates/hick-profile/tests/metrics.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-profile/tests/metrics.rs"><storycode:paste select="#f-0c82852f57d0-sampled_weights_preserve_threads_and_do_not_invent_calls"/><storycode:paste select="#f-0c82852f57d0-recursion_does_not_multiply_inclusive_stack_weight"/><storycode:paste select="#f-0c82852f57d0-evented_profiles_keep_self_and_overlapping_inclusive_costs_separate"/><storycode:paste select="#f-0c82852f57d0-v8_uses_microsecond_deltas_and_zero_based_locations"/><storycode:paste select="#f-0c82852f57d0-missing_v8_deltas_remain_samples_not_estimated_time"/><storycode:paste select="#f-0c82852f57d0-cprofile_retains_recursion_counts_and_inclusive_times"/><storycode:paste select="#f-0c82852f57d0-malformed_or_unknown_profiles_never_become_zero_cost"/><storycode:paste select="#f-0c82852f57d0-processed_firefox_preserves_symbolicated_rust_locations_and_sample_units"/><storycode:paste select="#f-0c82852f57d0-unconverted_firefox_profiles_are_unavailable_instead_of_zero"/></storycode:file>

<a id="output-0ac04607554d"></a>
## `crates/hickory-cli/src/performance/tests.rs`

Related contracts: [Profiles keep their measured scope](contract.md).

<a id="f-0ac04607554d-req"></a>
### `req`

<storycode:copy id="f-0ac04607554d-req">
// Guarantee: docs/guarantees/execution/profiles-keep-their-measured-scope.md
use super::*;
fn req(v: Value) -> Request {
    serde_json::from_value(v).unwrap()
}
</storycode:copy>
<a id="f-0ac04607554d-project"></a>
### `project`

<storycode:copy id="f-0ac04607554d-project">
fn project() -> (tempfile::TempDir, PathBuf) {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().canonicalize().unwrap();
    std::fs::write(root.join("app.py"), "def hot():\n    return 1\nhot()\n").unwrap();
    (t, root)
}
</storycode:copy>
<a id="f-0ac04607554d-report"></a>
### `report`

<storycode:copy id="f-0ac04607554d-report">
fn report(file: &str) -> String {
    json!({"elapsed_seconds":1,"functions":[{"name":"hot","file":file,"line":1,"total_calls":3,"primitive_calls":3,"self_seconds":0.1,"cumulative_seconds":0.2}]}).to_string()
}
</storycode:copy>
<a id="f-0ac04607554d-imports_keep_unknown_identity_and_original_evidence"></a>
### `imports_keep_unknown_identity_and_original_evidence`

<storycode:copy id="f-0ac04607554d-imports_keep_unknown_identity_and_original_evidence">
#[tokio::test]
async fn imports_keep_unknown_identity_and_original_evidence() {
    let (_t, root) = project();
    let run = dispatch(
        &root,
        req(json!({"action":"import","path":".","format":"cprofile-json","raw":report("app.py")})),
    )
    .await
    .unwrap();
    assert_eq!(run["source_state"], "unknown");
    assert!(run["sources"].as_object().unwrap().is_empty());
    assert!(!run["report"]["raw"].as_str().unwrap().is_empty());
    assert!(
        dispatch(
            &root,
            req(json!({"action":"compare","id":run["id"],"other":run["id"]}))
        )
        .await
        .is_err()
    );
}
</storycode:copy>
<a id="f-0ac04607554d-captured_hashes_bind_to_literate_source_and_stale_edits_remain_historical"></a>
### `captured_hashes_bind_to_literate_source_and_stale_edits_remain_historical`

<storycode:copy id="f-0ac04607554d-captured_hashes_bind_to_literate_source_and_stale_edits_remain_historical">
#[tokio::test]
async fn captured_hashes_bind_to_literate_source_and_stale_edits_remain_historical() {
    let (_t, root) = project();
    std::fs::write(
        root.join("app.md"),
        "# App\n<hick:file path=\"app.py\">\ndef hot():\n    return 1\nhot()\n</hick:file>\n",
    )
    .unwrap();
    let identity = Identity {
        collector: "test".into(),
        version: "1".into(),
        configuration: "function-profile".into(),
        scope: "".into(),
        source_hashes: snapshot(&root).unwrap(),
    };
    let run=dispatch(&root,req(json!({"action":"import","path":".","format":"cprofile-json","raw":report("app.py"),"identity":identity,"labels":["manual"]}))).await.unwrap();
    assert_eq!(run["mappings"]["app.py"]["1"]["path"], "app.md");
    assert_eq!(run["mappings"]["app.py"]["1"]["line"], 3);
    assert!(
        run["sources"]["app.md"]
            .as_str()
            .unwrap()
            .contains("def hot")
    );
    let page = dispatch(
        &root,
        req(json!({"action":"locations","id":run["id"],"limit":1})),
    )
    .await
    .unwrap();
    assert_eq!(page["rows"][0]["total_calls"], 3);
    assert!(page["rows"][0]["samples"].is_null());
    std::fs::write(root.join("app.py"), "edited\n").unwrap();
    assert_eq!(
        store::view(&root, run["id"].as_str().unwrap(), true).unwrap()["source_state"],
        "historical"
    );
}
</storycode:copy>
<a id="f-0ac04607554d-browser_generated_code_requires_exact_bytes_or_verified_source_map"></a>
### `browser_generated_code_requires_exact_bytes_or_verified_source_map`

<storycode:copy id="f-0ac04607554d-browser_generated_code_requires_exact_bytes_or_verified_source_map">
#[test]
fn browser_generated_code_requires_exact_bytes_or_verified_source_map() {
    let (_t, root) = project();
    let request = req(json!({"action":"browser-start"}));
    let mut record = fresh(&root, &root, &request, "manual-browser").unwrap();
    let raw=json!({"nodes":[{"id":1,"callFrame":{"functionName":"hot","url":format!("file://{}",root.join("app.py").display()),"lineNumber":0,"columnNumber":0,"scriptId":"1"}}],"startTime":0,"endTime":1000,"samples":[1],"timeDeltas":[1000]}).to_string();
    record.report = Some(hick_profile::parse("chrome-cpu", &raw).unwrap());
    let bad = BTreeMap::from([(
        "1".into(),
        browser::Script {
            url: format!("file://{}", root.join("app.py").display()),
            source: Some("transformed bytes".into()),
            ..Default::default()
        },
    )]);
    resolve::bind(&root, &mut record, Some(&bad)).unwrap();
    assert!(record.sources.is_empty());
    let good = BTreeMap::from([(
        "1".into(),
        browser::Script {
            url: format!("file://{}", root.join("app.py").display()),
            source: Some(std::fs::read_to_string(root.join("app.py")).unwrap()),
            ..Default::default()
        },
    )]);
    resolve::bind(&root, &mut record, Some(&good)).unwrap();
    assert!(record.sources.contains_key("app.py"));
}
</storycode:copy>
<a id="f-0ac04607554d-before_after_comparisons_preserve_revision_changes_and_missing_rows"></a>
### `before_after_comparisons_preserve_revision_changes_and_missing_rows`

<storycode:copy id="f-0ac04607554d-before_after_comparisons_preserve_revision_changes_and_missing_rows">
#[tokio::test]
async fn before_after_comparisons_preserve_revision_changes_and_missing_rows() {
    let (_t, root) = project();
    let mut a = fresh(&root, &root, &req(json!({"action":"run"})), "native").unwrap();
    a.identity.collector = "test".into();
    a.identity.version = "1".into();
    a.identity.configuration = "same workload".into();
    a.report = Some(hick_profile::parse("cprofile-json", &report("app.py")).unwrap());
    a.status = "completed".into();
    store::save(&root, &a).unwrap();
    let mut b = a.clone();
    b.id = "a".repeat(32);
    b.identity
        .source_hashes
        .insert("app.py".into(), "different revision".into());
    b.report.as_mut().unwrap().threads[0].rows.clear();
    store::save(&root, &b).unwrap();
    let comparison = dispatch(
        &root,
        req(json!({"action":"compare","id":a.id,"other":b.id})),
    )
    .await
    .unwrap();
    assert_eq!(comparison["same_sources"], false);
    assert!(comparison["rows"][0]["right"].is_null());
    assert!(comparison["rows"][0]["self_delta"].is_null());
    b.identity.configuration = "different sampling".into();
    store::save(&root, &b).unwrap();
    assert!(
        dispatch(
            &root,
            req(json!({"action":"compare","id":a.id,"other":b.id}))
        )
        .await
        .is_err()
    );
}
</storycode:copy>
<a id="f-0ac04607554d-unsafe_paths_and_remote_debugger_endpoints_are_refused"></a>
### `unsafe_paths_and_remote_debugger_endpoints_are_refused`

<storycode:copy id="f-0ac04607554d-unsafe_paths_and_remote_debugger_endpoints_are_refused">
#[tokio::test]
async fn unsafe_paths_and_remote_debugger_endpoints_are_refused() {
    let (_t, root) = project();
    assert!(
        dispatch(&root, req(json!({"action":"get","id":"../outside"})))
            .await
            .is_err()
    );
    assert!(
        dispatch(&root, req(json!({"action":"inspect","path":"../"})))
            .await
            .is_err()
    );
    assert!(
        browser::connect("http://example.com:9222", None)
            .await
            .is_err()
    );
    #[cfg(unix)]
    {
        let out = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(out.path(), root.join(".hick-cache")).unwrap();
        assert!(store::dir(&root).is_err());
    }
}
</storycode:copy>
<a id="f-0ac04607554d-native_profile_report_traversal_and_symlink_escape_are_refused_before_writes"></a>
### `native_profile_report_traversal_and_symlink_escape_are_refused_before_writes`

<storycode:copy id="f-0ac04607554d-native_profile_report_traversal_and_symlink_escape_are_refused_before_writes">
#[test]
fn native_profile_report_traversal_and_symlink_escape_are_refused_before_writes() {
    let (_t, root) = project();
    #[cfg(unix)]
    {
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.join("outside")).unwrap();
        let config = json!({"bad":{"argv":["not-run"],"format":"speedscope","report":"outside/new/report.json","collector":"test","version":"1"}});
        std::fs::write(root.join(".hick-profile.json"), config.to_string()).unwrap();
        let request = req(json!({"action":"run","path":".","profile":"bad"}));
        let mut run = fresh(&root, &root, &request, "native").unwrap();
        assert!(native::plan(&root, &root, &request, &mut run).is_err());
        assert!(!outside.path().join("new").exists());
    }
    std::fs::write(root.join(".hick-profile.json"),json!({"bad":{"argv":["not-run"],"format":"speedscope","report":"../outside/report.json","collector":"test","version":"1"}}).to_string()).unwrap();
    let request = req(json!({"action":"run","path":".","profile":"bad"}));
    let mut run = fresh(&root, &root, &request, "native").unwrap();
    assert!(native::plan(&root, &root, &request, &mut run).is_err());
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/performance/tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/performance/tests.rs"><storycode:paste select="#f-0ac04607554d-req"/><storycode:paste select="#f-0ac04607554d-project"/><storycode:paste select="#f-0ac04607554d-report"/><storycode:paste select="#f-0ac04607554d-imports_keep_unknown_identity_and_original_evidence"/><storycode:paste select="#f-0ac04607554d-captured_hashes_bind_to_literate_source_and_stale_edits_remain_historical"/><storycode:paste select="#f-0ac04607554d-browser_generated_code_requires_exact_bytes_or_verified_source_map"/><storycode:paste select="#f-0ac04607554d-before_after_comparisons_preserve_revision_changes_and_missing_rows"/><storycode:paste select="#f-0ac04607554d-unsafe_paths_and_remote_debugger_endpoints_are_refused"/><storycode:paste select="#f-0ac04607554d-native_profile_report_traversal_and_symlink_escape_are_refused_before_writes"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/performance/inline.ts](contract.md#output-3ec4aa2fc9ed)
- [crates/hickory-cli/src/performance/browser.rs](../../../mechanisms/crates/hickory-cli/src/performance/browser.rs.md#output-6728a647dd45)
- [crates/hickory-cli/src/performance/native.rs](../../../mechanisms/crates/hickory-cli/src/performance/native.rs.md#output-7c2a7d2033c5)
- [crates/hickory-cli/src/performance/resolve.rs](../../../mechanisms/crates/hickory-cli/src/performance/resolve.rs.md#output-78c84925a5fb)
- [crates/hickory-cli/src/performance/tests.rs](contract.md#output-0ac04607554d)
- [crates/hickory-cli/src/source_mapping.rs](../../../mechanisms/crates/hickory-cli/src/source_mapping.rs.md#output-bb86e57f10bc)

### Tests and fixtures

- [apps/web/src/performance/PerformanceTools.test.tsx](contract.md#output-9f4934c1bafd)
- [apps/web/src/performance/inline.test.ts](contract.md#output-da78f5045853)
- [apps/web/src/performance/selection.test.ts](contract.md#output-22cf5dd7fa52)
- [crates/hick-profile/tests/metrics.rs](contract.md#output-0c82852f57d0)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
