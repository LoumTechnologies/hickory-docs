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

export function failedRunBanner(run: Run): Banner {
  if (run.status === "stopped") return { kind: "fail", text: "Run stopped. Work already recorded is kept." };
  if (run.status !== "failed") return null;
  return { kind: "fail", text: run.error
    ? `Run failed: ${run.error}`
    : "Run failed — inspect cell output, expectation differences, and output drift." };
}

/** Agent terminal notices share the channel but have no document run receipt.
 * They must not settle a document's cells or manufacture a read-error banner. */
export function settleDocumentRun(runId: string, knownRuns: ReadonlySet<string>, settled: (run: Run) => void, failed: (error: unknown) => void): Promise<void> {
  return api.runStatus(runId).then(settled, error => { if (knownRuns.has(runId)) failed(error); });
}
