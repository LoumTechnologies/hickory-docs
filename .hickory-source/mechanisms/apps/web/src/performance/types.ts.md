<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/performance/types.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5f10c6622ff0"></a>
## `apps/web/src/performance/types.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-5f10c6622ff0-ProfileFrame"></a>
### `ProfileFrame`

<storycode:copy id="f-5f10c6622ff0-ProfileFrame">
export interface ProfileFrame { name: string; file: string | null; line: number | null; column: number | null; script_id: string | null }
</storycode:copy>
<a id="f-5f10c6622ff0-ProfileRow"></a>
### `ProfileRow`

<storycode:copy id="f-5f10c6622ff0-ProfileRow">
export interface ProfileRow { frame: ProfileFrame; self_weight: number; inclusive_weight: number; samples: number | null; total_calls: number | null; primitive_calls: number | null }
</storycode:copy>
<a id="f-5f10c6622ff0-ProfileScope"></a>
### `ProfileScope`

<storycode:copy id="f-5f10c6622ff0-ProfileScope">
export interface ProfileScope { name: string; unit: string; aggregation: string; interval_unit: string; clock: string; interval_start: number; interval_end: number; measured_weight: number; rows: ProfileRow[] }
</storycode:copy>
<a id="f-5f10c6622ff0-PerformanceRun"></a>
### `PerformanceRun`

<storycode:copy id="f-5f10c6622ff0-PerformanceRun">
export interface PerformanceRun {
  id: string; mode: string; status: string; started_ms: number; labels: string[];
  source_state: "current" | "historical" | "unknown"; source_changed: boolean;
  identity: { collector: string; version: string; configuration: string; scope: string; source_hashes: Record<string, string> };
  report: { format: string; threads: ProfileScope[]; raw: string; limitations: string[] } | null;
  mappings: Record<string, Record<string, { path: string; line: number }>>;
  sources: Record<string, string>; environment: unknown; output: string; error: string | null;
}
</storycode:copy>

### Assemble `apps/web/src/performance/types.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/performance/types.ts"><storycode:paste select="#f-5f10c6622ff0-ProfileFrame"/><storycode:paste select="#f-5f10c6622ff0-ProfileRow"/><storycode:paste select="#f-5f10c6622ff0-ProfileScope"/><storycode:paste select="#f-5f10c6622ff0-PerformanceRun"/></storycode:file>

</storycode:doc>
