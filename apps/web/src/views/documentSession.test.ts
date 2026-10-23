// The run channel carries three shapes: transcript events, terminal run
// statuses, and the up-loop's files_changed notice (watch.rs::
// notify_files_changed). The session's handler must route the third to the
// outputs refetch — before this guard existed it fell into the terminal-
// status branch, cleared "running" state for a run that never was, and
// fetched /api/runs/undefined.

import { describe, expect, it } from "vitest";
import type { RunWsMessage } from "../api/types";
import { isFilesChanged } from "./documentSession";

describe("isFilesChanged", () => {
  it("recognises the up-loop's exact payload", () => {
    const msg: RunWsMessage = { files_changed: true, doc: "doc-1" };
    expect(isFilesChanged(msg)).toBe(true);
  });

  it("rejects transcript events and terminal statuses", () => {
    const transcript: RunWsMessage = {
      run_id: "r1",
      exec_id: "e1",
      event: { t: 0, kind: "out", data: "hi" },
    };
    const terminal: RunWsMessage = { run_id: "r1", status: "ok" };
    expect(isFilesChanged(transcript)).toBe(false);
    expect(isFilesChanged(terminal)).toBe(false);
  });
});

import { completedRunBlocks, failedRunBanner } from "./documentSession";
import type { Block, Run } from "../api/types";

describe("terminal execution failures", () => {
  const cell = (id: string, transcript: Extract<Block, {kind: "exec"}>["transcript"] = []): Block => ({
    kind: "exec", id, command: "printf before; exit 9", container: "shell", span: [0, 1], status: "unrecorded", transcript,
  });
  const failed = (error: string): Run => ({ id: "r1", started_at: "now", status: "failed", blocks: [], error });

  it.each(["cell failed with exit code 9", "cell exceeded its timeout of 1s"])("keeps partial output and displays the execution diagnostic: %s", error => {
    // The server sends an empty receipt when execution itself aborts. A
    // re-render here would discard 'before' and return the cell to never run.
    const output = [{ t: 0, kind: "out" as const, data: "before" }];
    const blocks = completedRunBlocks(failed(error), [cell("started", output), cell("not-reached")], new Set(["started"]));
    expect(blocks?.[0]).toMatchObject({ status: "failed", transcript: output });
    expect(blocks?.[1]).toMatchObject({ status: "unrecorded", transcript: [] });
    expect(failedRunBanner(failed(error))).toEqual({ kind: "fail", text: `Run failed: ${error}` });
  });

  it("preserves an earlier successful cell when a later cell aborts", () => {
    const blocks = completedRunBlocks(failed("timeout"), [{ ...cell("done", [{ t: 0, kind: "exit", code: 0 }]), status: "ok" } as Block, cell("aborted")], new Set(["done", "aborted"]));
    expect(blocks?.[0]).toMatchObject({ status: "ok" });
    expect(blocks?.[1]).toMatchObject({ status: "failed" });
  });

  it("uses the authoritative receipt for expectation failures and successful reruns", () => {
    const receipt = [cell("new")];
    expect(completedRunBlocks({ ...failed(""), blocks: receipt }, [cell("old")], new Set())).toBe(receipt);
    expect(failedRunBanner({ ...failed(""), status: "ok" })).toBeNull();
    expect(failedRunBanner(failed(""))?.text).toContain("inspect cell output");
  });
});

import { api } from "../api/client";
import { settleDocumentRun } from "./documentSession";
import { vi } from "vitest";

it("ignores an agent terminal without a run receipt while a document remains pending, then settles the real document run", async () => {
  let documentPending = true;
  const known = new Set(["document-run"]);
  const settled = vi.fn(() => { documentPending = false; });
  const failed = vi.fn();
  const status = vi.spyOn(api, "runStatus");
  try {
    // Agent status rides this same channel but /api/runs has no such record.
    status.mockRejectedValueOnce(new Error("no such run in this session"));
    await settleDocumentRun("agent-turn", known, settled, failed);
    expect(documentPending).toBe(true);
    expect(settled).not.toHaveBeenCalled();
    expect(failed).not.toHaveBeenCalled();
    const receipt: Run = { id: "document-run", status: "failed", started_at: "now", blocks: [], error: "timeout" };
    status.mockResolvedValueOnce(receipt);
    await settleDocumentRun("document-run", known, settled, failed);
    expect(documentPending).toBe(false);
    expect(settled).toHaveBeenCalledWith(receipt);
    // A known document run genuinely losing its receipt is still actionable.
    status.mockRejectedValueOnce(new Error("document receipt unavailable"));
    await settleDocumentRun("document-run", known, settled, failed);
    expect(failed).toHaveBeenCalledWith(expect.objectContaining({ message: "document receipt unavailable" }));
  } finally { status.mockRestore(); }
});
