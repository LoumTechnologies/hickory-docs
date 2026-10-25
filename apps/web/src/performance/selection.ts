import type { ObservationReading } from "../debug/observations";
import type { PerformanceRun } from "./types";
let selected: { run: PerformanceRun; thread: number } | null = null;
const event = "hickory:performance-selection";
export function selectPerformance(run: PerformanceRun | null, thread = 0) { selected = run ? { run, thread } : null; window.dispatchEvent(new Event(event)); }
export function selectedPerformance() { return selected; }
export function onPerformanceSelection(callback: () => void) { window.addEventListener(event, callback); return () => window.removeEventListener(event, callback); }
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
