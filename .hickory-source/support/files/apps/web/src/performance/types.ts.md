<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/performance/types.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `4485859a6d8a733a1a5c8da1ba2492aefab1f90dec4a3aa03cfb6cacb510e8ef`.

<book:entry path="apps/web/src/performance/types.ts">
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
</book:entry>

</book:doc>
