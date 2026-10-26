<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/documentRunResults.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-cb31e1d67a63"></a>
## `apps/web/src/views/documentRunResults.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-cb31e1d67a63-completedRunBlocks"></a>
### `completedRunBlocks`

<storycode:copy id="f-cb31e1d67a63-completedRunBlocks">
import { api } from "../api/client";
import type { Block, Run } from "../api/types";
import type { Banner } from "./documentSession";

/** Failed execution may have no woven receipt. Keep the live transcript in
 * that case instead of replacing it with a fresh, unexecuted render. Only
 * cells actually started by this run can be marked failed. */
export function completedRunBlocks(run: Run, live: Block[] | null, started: ReadonlySet<string>, requested: ReadonlySet<string> = new Set<string>()): Block[] | null {
  if (run.blocks?.length) return run.blocks;
  if (!live || run.status !== "failed") return live;
  return live.map((block): Block => {
    if (block.kind !== "exec") return block;
    if (started.has(block.id)) return block.transcript?.some(event => event.kind === "exit" && event.code === 0)
      ? block : { ...block, status: "failed" };
    return requested.has(block.id)
      ? { ...block, runError: run.error ?? "The document run failed without returning per-cell results." }
      : block;
  });
}

</storycode:copy>
<a id="f-cb31e1d67a63-failedRunBanner"></a>
### `failedRunBanner`

<storycode:copy id="f-cb31e1d67a63-failedRunBanner">
export function failedRunBanner(run: Run): Banner {
  if (run.status === "stopped") return { kind: "fail", text: "Run stopped. Work already recorded is kept." };
  if (run.status !== "failed") return null;
  return { kind: "fail", text: run.error
    ? `Run failed: ${run.error}`
    : "Run failed — inspect cell output, expectation differences, and output drift." };
}

</storycode:copy>
<a id="f-cb31e1d67a63-settleDocumentRun"></a>
### `settleDocumentRun`

Agent terminal notices share the channel but have no document run receipt.
They must not settle a document's cells or manufacture a read-error banner.

<storycode:copy id="f-cb31e1d67a63-settleDocumentRun">
/** Agent terminal notices share the channel but have no document run receipt.
 * They must not settle a document's cells or manufacture a read-error banner. */
export function settleDocumentRun(runId: string, knownRuns: ReadonlySet<string>, settled: (run: Run) => void, failed: (error: unknown) => void): Promise<void> {
  return api.runStatus(runId).then(settled, error => { if (knownRuns.has(runId)) failed(error); });
}
</storycode:copy>

### Assemble `apps/web/src/views/documentRunResults.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/documentRunResults.ts"><storycode:paste select="#f-cb31e1d67a63-completedRunBlocks"/><storycode:paste select="#f-cb31e1d67a63-failedRunBanner"/><storycode:paste select="#f-cb31e1d67a63-settleDocumentRun"/></storycode:file>

</storycode:doc>
