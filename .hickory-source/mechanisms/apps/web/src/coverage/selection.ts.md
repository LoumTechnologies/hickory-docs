<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/coverage/selection.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ce9e9970e5a9"></a>
## `apps/web/src/coverage/selection.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-ce9e9970e5a9-selected"></a>
### `selected`

<storycode:copy id="f-ce9e9970e5a9-selected">
import type { CoverageRun } from "./types";
let selected: CoverageRun | null = null;
</storycode:copy>
<a id="f-ce9e9970e5a9-event"></a>
### `event`

<storycode:copy id="f-ce9e9970e5a9-event">
const event = "hickory:coverage-selection";
</storycode:copy>
<a id="f-ce9e9970e5a9-selectedCoverage"></a>
### `selectedCoverage`

<storycode:copy id="f-ce9e9970e5a9-selectedCoverage">
export function selectedCoverage() { return selected; }
</storycode:copy>
<a id="f-ce9e9970e5a9-selectCoverage"></a>
### `selectCoverage`

<storycode:copy id="f-ce9e9970e5a9-selectCoverage">
export function selectCoverage(run: CoverageRun | null) { selected = run; window.dispatchEvent(new Event(event)); }
</storycode:copy>
<a id="f-ce9e9970e5a9-onCoverageSelection"></a>
### `onCoverageSelection`

<storycode:copy id="f-ce9e9970e5a9-onCoverageSelection">
export function onCoverageSelection(callback: () => void) { window.addEventListener(event, callback); return () => window.removeEventListener(event, callback); }
</storycode:copy>
<a id="f-ce9e9970e5a9-coveredLocations"></a>
### `coveredLocations`

<storycode:copy id="f-ce9e9970e5a9-coveredLocations">
export function coveredLocations(run: CoverageRun, path: string): { line: number; count: number; originalPath: string; originalLine: number; countsAvailable: boolean }[] {
  const out: ReturnType<typeof coveredLocations> = [];
  for (const [file, data] of Object.entries(run.evidence?.report.files ?? {})) {
    for (const [n, count] of Object.entries(data.lines)) {
      if (file === path) out.push({ line: +n, count, countsAvailable: data.line_counts, originalPath: file, originalLine: +n });
      else if (run.mappings[file]?.[n]?.path === path) out.push({ line: run.mappings[file][n].line, count, countsAvailable: data.line_counts, originalPath: file, originalLine: +n });
    }
  }
  return out;
}
</storycode:copy>

### Assemble `apps/web/src/coverage/selection.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/coverage/selection.ts"><storycode:paste select="#f-ce9e9970e5a9-selected"/><storycode:paste select="#f-ce9e9970e5a9-event"/><storycode:paste select="#f-ce9e9970e5a9-selectedCoverage"/><storycode:paste select="#f-ce9e9970e5a9-selectCoverage"/><storycode:paste select="#f-ce9e9970e5a9-onCoverageSelection"/><storycode:paste select="#f-ce9e9970e5a9-coveredLocations"/></storycode:file>

</storycode:doc>
