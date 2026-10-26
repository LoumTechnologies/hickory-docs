<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/coverage/types.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f19aeadf1793"></a>
## `apps/web/src/coverage/types.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-f19aeadf1793-CoverageFile"></a>
### `CoverageFile`

<storycode:copy id="f-f19aeadf1793-CoverageFile">
export interface CoverageFile { line_counts: boolean; branch_totals: Record<string, [number, number]>; lines: Record<string, number>; branches: { id: string; line: number; count: number | null; covered: boolean }[] | null; excluded: number[] }
</storycode:copy>
<a id="f-f19aeadf1793-CoverageIdentity"></a>
### `CoverageIdentity`

<storycode:copy id="f-f19aeadf1793-CoverageIdentity">
export interface CoverageIdentity { collector: string; version: string; configuration: string; scope: string; source_hashes: Record<string, string> }
</storycode:copy>
<a id="f-f19aeadf1793-CoverageRun"></a>
### `CoverageRun`

<storycode:copy id="f-f19aeadf1793-CoverageRun">
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
</storycode:copy>
<a id="f-f19aeadf1793-CoverageComparison"></a>
### `CoverageComparison`

<storycode:copy id="f-f19aeadf1793-CoverageComparison">
export interface CoverageComparison { shared: string[]; left_only: string[]; right_only: string[]; missed: string[]; denominator: number }
</storycode:copy>

### Assemble `apps/web/src/coverage/types.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/coverage/types.ts"><storycode:paste select="#f-f19aeadf1793-CoverageFile"/><storycode:paste select="#f-f19aeadf1793-CoverageIdentity"/><storycode:paste select="#f-f19aeadf1793-CoverageRun"/><storycode:paste select="#f-f19aeadf1793-CoverageComparison"/></storycode:file>

</storycode:doc>
