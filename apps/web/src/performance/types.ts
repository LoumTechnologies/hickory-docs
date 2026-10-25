export interface ProfileFrame { name: string; file: string | null; line: number | null; column: number | null; script_id: string | null }
export interface ProfileRow { frame: ProfileFrame; self_weight: number; inclusive_weight: number; samples: number | null; total_calls: number | null; primitive_calls: number | null }
export interface ProfileScope { name: string; unit: string; aggregation: string; interval_unit: string; clock: string; interval_start: number; interval_end: number; measured_weight: number; rows: ProfileRow[] }
export interface PerformanceRun {
  id: string; mode: string; status: string; started_ms: number; labels: string[];
  source_state: "current" | "historical" | "unknown"; source_changed: boolean;
  identity: { collector: string; version: string; configuration: string; scope: string; source_hashes: Record<string, string> };
  report: { format: string; threads: ProfileScope[]; raw: string; limitations: string[] } | null;
  mappings: Record<string, Record<string, { path: string; line: number }>>;
  sources: Record<string, string>; environment: unknown; output: string; error: string | null;
}
