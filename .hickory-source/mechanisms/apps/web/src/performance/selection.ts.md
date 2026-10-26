<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/performance/selection.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7c4c71cf2d37"></a>
## `apps/web/src/performance/selection.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-7c4c71cf2d37-selected"></a>
### `selected`

<storycode:copy id="f-7c4c71cf2d37-selected">
import type { ObservationReading } from "../debug/observations";
import type { PerformanceRun } from "./types";
let selected: { run: PerformanceRun; thread: number } | null = null;
</storycode:copy>
<a id="f-7c4c71cf2d37-event"></a>
### `event`

<storycode:copy id="f-7c4c71cf2d37-event">
const event = "hickory:performance-selection";
</storycode:copy>
<a id="f-7c4c71cf2d37-selectPerformance"></a>
### `selectPerformance`

<storycode:copy id="f-7c4c71cf2d37-selectPerformance">
export function selectPerformance(run: PerformanceRun | null, thread = 0) { selected = run ? { run, thread } : null; window.dispatchEvent(new Event(event)); }
</storycode:copy>
<a id="f-7c4c71cf2d37-selectedPerformance"></a>
### `selectedPerformance`

<storycode:copy id="f-7c4c71cf2d37-selectedPerformance">
export function selectedPerformance() { return selected; }
</storycode:copy>
<a id="f-7c4c71cf2d37-onPerformanceSelection"></a>
### `onPerformanceSelection`

<storycode:copy id="f-7c4c71cf2d37-onPerformanceSelection">
export function onPerformanceSelection(callback: () => void) { window.addEventListener(event, callback); return () => window.removeEventListener(event, callback); }
</storycode:copy>
<a id="f-7c4c71cf2d37-profileReading"></a>
### `profileReading`

Reuse the shared source-anchored observation/context contract. Costs anchor functions.

<storycode:copy id="f-7c4c71cf2d37-profileReading">
/** Reuse the shared source-anchored observation/context contract. Costs anchor functions. */
export function profileReading(run: PerformanceRun, thread: number, path: string): ObservationReading | null {
  const source = run.sources[path], scope = run.report?.threads[thread];
  if (source === undefined || !scope) return null;
  const bytes = new TextEncoder().encode(source), revision = run.identity.source_hashes[path];
  if (!revision) return null;
  const reading: ObservationReading = { schema_version: 1, source: { id: `${run.id}:${path}`, uri: path, revision, bytes: [...bytes] }, scope: { run: run.id, stop: `profile:${thread}`, frame: null, invocation: null }, locations: [], annotations: [], unsupported: [...(run.report?.limitations ?? []), "Function location anchors are not per-line timings", "No invocation identity inferred"] };
  const lines = source.split("\n"), offsets = [0];
  for (let i = 0; i < lines.length; i++) offsets.push(offsets[i] + new TextEncoder().encode(lines[i] + "\n").length);
  scope.rows.forEach((row, index) => {
    const file = row.frame.file, originalLine = row.frame.line;
    if (!file || !originalLine) return;
    const mapped = run.mappings[file]?.[originalLine];
    const line = file === path ? originalLine : mapped?.path === path ? mapped.line : null;
    if (!line || line > lines.length) return;
    const id = `profile:${run.id}:${thread}:L${line}`;
    if (!reading.locations.some(item => item.id === id)) reading.locations.push({ id, revision, line: line - 1, location: { source: reading.source.id, bytes: { start: offsets[line - 1], end: offsets[line - 1] + new TextEncoder().encode(lines[line - 1].replace(/\r$/, "")).length }, precision: "adapter_reported" } });
    reading.annotations.push({ evidence: `${run.id}:profile:${thread}:${index}`, provider: run.identity.collector, payload_schema: "hick-performance-v1", location: id, scope: reading.scope, payload: { kind: "profile", row, unit: scope.unit, aggregation: scope.aggregation, interval: [scope.interval_start, scope.interval_end], interval_unit: scope.interval_unit ?? "unknown", clock: scope.clock ?? "unknown", measured_weight: scope.measured_weight, labels: run.labels, source_state: run.source_state, environment: run.environment, collector_version: run.identity.version } });
  });
  return reading;
}
</storycode:copy>

### Assemble `apps/web/src/performance/selection.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/performance/selection.ts"><storycode:paste select="#f-7c4c71cf2d37-selected"/><storycode:paste select="#f-7c4c71cf2d37-event"/><storycode:paste select="#f-7c4c71cf2d37-selectPerformance"/><storycode:paste select="#f-7c4c71cf2d37-selectedPerformance"/><storycode:paste select="#f-7c4c71cf2d37-onPerformanceSelection"/><storycode:paste select="#f-7c4c71cf2d37-profileReading"/></storycode:file>

</storycode:doc>
