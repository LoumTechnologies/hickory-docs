<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/runtimeObservations.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `254cc095403f26001f087859f2f70893e39dea74f8aa569e60df9fceaaf2308b`.

<book:entry path="apps/web/src/debug/runtimeObservations.ts">
/** Wire-compatible subset of hick-observe v1. Offsets are UTF-8 bytes. */
export interface RuntimeLocation {
  source: string; block: string | null; bytes: { start: number; end: number } | null;
  precision: "exact" | "adapter_reported" | "block_only" | "unknown";
}
export interface RuntimeEvent {
  id: string; provider: string; block: string | null;
  invocation: string | null; parent_invocation: string | null;
  thread: string | null; task: string | null;
  time: { domain: string; unit: string; value: number } | null;
  emission: RuntimeLocation | null; callers: RuntimeLocation[];
  observation: { kind: "output"; stream: string; category: string; bytes: number[] } |
    { kind: "exception"; type_name: string; message: string;
      handling: "throwing" | "caught" | "unhandled" | "unknown";
      throwing: RuntimeLocation | null; catching: RuntimeLocation | null } |
    { kind: "log"; level: string; message: string; fields: Record<string, unknown> } |
    { kind: "debugger"; reason: string; details: unknown } |
    { kind: "profile"; metric: string; value: number; unit: string;
      interval: { start: { domain: string; unit: string; value: number }; end: { domain: string; unit: string; value: number }; scope: string };
      aggregation: string; samples: number | null; function?: { name: string; file: string; line: number; primitive_calls: number; total_calls: number } | null };
}
export interface RuntimeRun {
  schema_version: 1; id: string; started_unix_ms: number;
  status: "running" | "completed" | "failed" | "partial" | "cancelled";
  pinned: boolean; sources: { id: string; uri: string; revision: string; bytes: number[] }[];
  events: RuntimeEvent[]; truncated: boolean; missing_artifacts: string[]; detail: string | null;
}

export function observationText(observation: RuntimeEvent["observation"]): string {
  switch (observation.kind) {
    case "output": return new TextDecoder().decode(new Uint8Array(observation.bytes));
    case "exception": return `${observation.handling} ${observation.type_name}: ${observation.message}\n`;
    case "log": return `${observation.level}: ${observation.message}\n`;
    case "profile": return `${observation.function?.name ?? "unresolved function"} · calls ${observation.function?.total_calls ?? "unknown"} (primitive ${observation.function?.primitive_calls ?? "unknown"}) · ${observation.metric}: ${observation.value} ${observation.unit} · ${observation.aggregation} · scope ${observation.interval.scope}\n`;
    case "debugger": return `${observation.reason}: ${JSON.stringify(observation.details)}\n`;
  }
}
</book:entry>

</book:doc>
