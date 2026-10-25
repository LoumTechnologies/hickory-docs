<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/executionEvidence.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A run result is distinct from what was checked](../../../../../contract.md)

Captured SHA-256: `2a7794496a7f4a6877dc6737723ff41dbc3739d9e1c0f2b0f077eb55c105d8b6`.

<book:entry path="apps/web/src/lib/executionEvidence.ts">
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
</book:entry>

</book:doc>
