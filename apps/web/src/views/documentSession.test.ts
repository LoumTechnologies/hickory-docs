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
