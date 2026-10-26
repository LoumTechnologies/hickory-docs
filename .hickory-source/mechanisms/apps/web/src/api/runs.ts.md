<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/api/runs.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d90062976539"></a>
## `apps/web/src/api/runs.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-d90062976539-RunStatus"></a>
### `RunStatus`

<storycode:copy id="f-d90062976539-RunStatus">
import type { Block } from "./types";

export type RunStatus = "queued" | "running" | "ok" | "failed" | "stopped";

</storycode:copy>
<a id="f-d90062976539-Run"></a>
### `Run`

<storycode:copy id="f-d90062976539-Run">
export interface Run {
  id: string;
  status: RunStatus;
  started_at: string;
  /** Execution/setup failure, including nonzero exits and timeouts. */
  error?: string | null;
  /**
   * The run's own block model — the same shape `/render` returns, with the
   * transcripts this run produced.
   *
   * Was typed as `{exec_id, status, transcript}[]`, which the server has
   * never sent: it stores `block_model_json(&run)["blocks"]`. The mistyping
   * was invisible because nothing read the field.
   */
  blocks: Block[];
}
</storycode:copy>

### Assemble `apps/web/src/api/runs.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/api/runs.ts"><storycode:paste select="#f-d90062976539-RunStatus"/><storycode:paste select="#f-d90062976539-Run"/></storycode:file>

</storycode:doc>
