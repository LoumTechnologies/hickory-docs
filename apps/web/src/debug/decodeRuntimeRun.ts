import type { RuntimeRun } from "./runtimeObservations";
import { sha256 } from "../lib/reviews";
const record = (value: unknown): value is Record<string, unknown> => !!value && typeof value === "object" && !Array.isArray(value);
const text = (value: unknown): value is string => typeof value === "string";
const number = (value: unknown): value is number => typeof value === "number" && Number.isFinite(value);
const natural = (value: unknown) => number(value) && Number.isSafeInteger(value) && value >= 0;
const nullableText = (value: unknown) => value === null || text(value);
const bytes = (value: unknown): value is number[] => Array.isArray(value) && value.every(byte => natural(byte) && byte <= 255);
const keys = (value: Record<string, unknown>, allowed: string[]) => Object.keys(value).every(key => allowed.includes(key));
const clock = (value: unknown) => record(value) && text(value.domain) && text(value.unit) && number(value.value);
function location(value: unknown): boolean {
  return value === null || record(value) && text(value.source) && nullableText(value.block)
    && ["exact", "adapter_reported", "block_only", "unknown"].includes(String(value.precision))
    && (value.bytes === null || record(value.bytes) && natural(value.bytes.start) && natural(value.bytes.end) && Number(value.bytes.end) >= Number(value.bytes.start));
}
function observation(value: unknown): boolean {
  if (!record(value)) return false;
  switch (value.kind) {
    case "output": return keys(value, ["kind", "stream", "category", "bytes"]) && text(value.stream) && text(value.category) && bytes(value.bytes);
    case "exception": return keys(value, ["kind", "type_name", "message", "handling", "throwing", "catching"]) && text(value.type_name) && text(value.message) && ["throwing", "caught", "unhandled", "unknown"].includes(String(value.handling)) && location(value.throwing) && location(value.catching);
    case "log": return keys(value, ["kind", "level", "message", "fields"]) && text(value.level) && text(value.message) && record(value.fields);
    case "debugger": return keys(value, ["kind", "reason", "details"]) && text(value.reason) && Object.hasOwn(value, "details");
    case "profile": return keys(value, ["kind", "metric", "value", "unit", "interval", "aggregation", "samples", "function"]) && text(value.metric) && number(value.value) && text(value.unit) && text(value.aggregation)
      && (value.samples === null || natural(value.samples)) && record(value.interval) && text(value.interval.scope) && clock(value.interval.start) && clock(value.interval.end)
      && (value.function == null || record(value.function) && text(value.function.name) && text(value.function.file) && natural(value.function.line) && natural(value.function.primitive_calls) && natural(value.function.total_calls));
    default: return false;
  }
}
/** Structural/integrity checks for explicitly imported recorded data. Hashes do
 * not authenticate its producer, establish execution, or certify current source. */
export async function decodeRuntimeRun(json: string): Promise<RuntimeRun> {
  if (new TextEncoder().encode(json).length > 2 * 1024 * 1024) throw Error("Recorded runtime import exceeds 2 MiB");
  const value: unknown = JSON.parse(json);
  const fail = () => { throw Error("Invalid recorded runtime evidence"); };
  if (!record(value) || !keys(value, ["schema_version", "id", "started_unix_ms", "status", "pinned", "sources", "events", "truncated", "missing_artifacts", "detail"])
    || value.schema_version !== 1 || !text(value.id) || !value.id || !natural(value.started_unix_ms) || !["running", "completed", "failed", "partial", "cancelled"].includes(String(value.status))
    || typeof value.pinned !== "boolean" || typeof value.truncated !== "boolean" || !nullableText(value.detail) || !Array.isArray(value.missing_artifacts) || !value.missing_artifacts.every(text)
    || !Array.isArray(value.sources) || !value.sources.length || value.sources.length > 128 || !Array.isArray(value.events) || value.events.length > 100000) return fail();
  const sourceIds = new Set<string>(), eventIds = new Set<string>();
  for (const source of value.sources) {
    if (!record(source) || !keys(source, ["id", "uri", "revision", "bytes"]) || !text(source.id) || !source.id || sourceIds.has(source.id) || !text(source.uri) || !text(source.revision) || !bytes(source.bytes)) return fail();
    sourceIds.add(source.id);
    const decoded = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(new Uint8Array(source.bytes));
    if (source.revision.replace(/^sha256:/, "") !== await sha256(decoded)) throw Error("Recorded source hash does not match its captured bytes");
  }
  for (const event of value.events) {
    if (!record(event) || !keys(event, ["id", "provider", "block", "invocation", "parent_invocation", "thread", "task", "time", "emission", "callers", "observation"])
      || !text(event.id) || !event.id || eventIds.has(event.id) || !text(event.provider) || ![event.block, event.invocation, event.parent_invocation, event.thread, event.task].every(nullableText)
      || event.time !== null && !clock(event.time) || !location(event.emission) || !Array.isArray(event.callers) || !event.callers.every(location) || !observation(event.observation)) return fail();
    eventIds.add(event.id);
  }
  return value as unknown as RuntimeRun;
}
