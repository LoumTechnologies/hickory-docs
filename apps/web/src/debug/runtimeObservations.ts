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
      throwing: RuntimeLocation | null; catching: RuntimeLocation | null };
}
export interface RuntimeRun {
  schema_version: 1; id: string; started_unix_ms: number;
  status: "running" | "completed" | "failed" | "partial" | "cancelled";
  pinned: boolean; sources: { id: string; uri: string; revision: string; bytes: number[] }[];
  events: RuntimeEvent[]; truncated: boolean; missing_artifacts: string[]; detail: string | null;
}
