import type { ExecStatus } from "../api/types";

/** A process result is distinct from matching an expectation. */
export function executionEvidence(status: ExecStatus = "unrecorded", running = false) {
  if (running) return { label: "Running…", detail: "Execution is in progress; there is no verdict yet." };
  switch (status) {
    case "ok": return { label: "Recorded · run succeeded", detail: "The last run succeeded. An expectation match is reported separately." };
    case "failed": return { label: "Recorded · run or check failed", detail: "Inspect the transcript and any expectation diff for the failure." };
    case "stale": return { label: "Stale recording", detail: "Inputs changed since this recording. Run again to obtain current evidence." };
    case "unrecorded": return { label: "Unrecorded · never run", detail: "No recording is available; displayed content is not execution evidence." };
  }
}
