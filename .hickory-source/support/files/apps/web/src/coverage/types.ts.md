<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/coverage/types.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `222f988981541f1e4beb5485246e85f08c5ee8121441d089622d1f2a4d16a434`.

<book:entry path="apps/web/src/coverage/types.ts">
export interface CoverageFile { line_counts: boolean; branch_totals: Record<string, [number, number]>; lines: Record<string, number>; branches: { id: string; line: number; count: number | null; covered: boolean }[] | null; excluded: number[] }
export interface CoverageIdentity { collector: string; version: string; configuration: string; scope: string; source_hashes: Record<string, string> }
export interface CoverageRun {
  id: string; mode: string; status: string; started_ms: number; project: string; language: string;
  labels: string[]; group: string | null; source_state: "current" | "historical" | "unknown";
  identity: CoverageIdentity; argv: string[]; output: string; error: string | null; exit_code: number | null;
  tests: { id: string; status: string; output: string; path: string | null; line: number | null; labels: string[] }[];
  evidence: { identity: CoverageIdentity; report: { format: string; files: Record<string, CoverageFile>; raw: string }; attribution: string; labels: string[]; tests: string[] } | null;
  metadata: { groups: Record<string, { selectors: string[]; labels: string[] }>; manual: Record<string, unknown>; tests: Record<string, string[]> };
  source_changed_during_run: boolean; manual_verdict: string | null; notes: string; checkpoints: string[];
  parent: string | null;
  mappings: Record<string, Record<string, { path: string; line: number }>>;
}
export interface CoverageComparison { shared: string[]; left_only: string[]; right_only: string[]; missed: string[]; denominator: number }
</book:entry>

</book:doc>
