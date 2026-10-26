<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/runtimeObservations.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8d4ceffd43b3"></a>
## `apps/web/src/debug/runtimeObservations.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-8d4ceffd43b3-RuntimeLocation"></a>
### `RuntimeLocation`

Wire-compatible subset of hick-observe v1. Offsets are UTF-8 bytes.

<storycode:copy id="f-8d4ceffd43b3-RuntimeLocation">
/** Wire-compatible subset of hick-observe v1. Offsets are UTF-8 bytes. */
export interface RuntimeLocation {
  source: string; block: string | null; bytes: { start: number; end: number } | null;
  precision: "exact" | "adapter_reported" | "block_only" | "unknown";
}
</storycode:copy>
<a id="f-8d4ceffd43b3-RuntimeEvent"></a>
### `RuntimeEvent`

<storycode:copy id="f-8d4ceffd43b3-RuntimeEvent">
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
</storycode:copy>
<a id="f-8d4ceffd43b3-RuntimeRun"></a>
### `RuntimeRun`

<storycode:copy id="f-8d4ceffd43b3-RuntimeRun">
export interface RuntimeRun {
  schema_version: 1; id: string; started_unix_ms: number;
  status: "running" | "completed" | "failed" | "partial" | "cancelled";
  pinned: boolean; sources: { id: string; uri: string; revision: string; bytes: number[] }[];
  events: RuntimeEvent[]; truncated: boolean; missing_artifacts: string[]; detail: string | null;
}

</storycode:copy>
<a id="f-8d4ceffd43b3-observationText"></a>
### `observationText`

<storycode:copy id="f-8d4ceffd43b3-observationText">
export function observationText(observation: RuntimeEvent["observation"]): string {
  switch (observation.kind) {
    case "output": return new TextDecoder().decode(new Uint8Array(observation.bytes));
    case "exception": return `${observation.handling} ${observation.type_name}: ${observation.message}\n`;
    case "log": return `${observation.level}: ${observation.message}\n`;
    case "profile": return `${observation.function?.name ?? "unresolved function"} · calls ${observation.function?.total_calls ?? "unknown"} (primitive ${observation.function?.primitive_calls ?? "unknown"}) · ${observation.metric}: ${observation.value} ${observation.unit} · ${observation.aggregation} · scope ${observation.interval.scope}\n`;
    case "debugger": return `${observation.reason}: ${JSON.stringify(observation.details)}\n`;
  }
}
</storycode:copy>

### Assemble `apps/web/src/debug/runtimeObservations.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/runtimeObservations.ts"><storycode:paste select="#f-8d4ceffd43b3-RuntimeLocation"/><storycode:paste select="#f-8d4ceffd43b3-RuntimeEvent"/><storycode:paste select="#f-8d4ceffd43b3-RuntimeRun"/><storycode:paste select="#f-8d4ceffd43b3-observationText"/></storycode:file>

</storycode:doc>
