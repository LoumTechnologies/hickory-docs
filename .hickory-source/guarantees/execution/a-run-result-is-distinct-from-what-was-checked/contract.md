<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A run result is distinct from what was checked

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a cell, diagram, or source-backed AI explanation, when status is
rendered, then successful execution does not alone claim verified output or
semantic correctness. Matching expected stdout is reported separately. Diagrams
report passed, failed, or not-yet-verified assertions; stale or absent execution
evidence cannot report passing assertions. Stale explanations identify source
change separately from execution verdicts. The action rail distinguishes stale
recordings from cells that have never run.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-run-result-is-distinct-from-what-was-checked.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-b0293590899e"></a>
## `apps/web/src/components/StatusChip.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A run result is distinct from what was checked](contract.md).

<a id="f-b0293590899e-StatusChip"></a>
### `StatusChip`

<storycode:copy id="f-b0293590899e-StatusChip">
import type { ExecStatus } from "../api/types";
import { executionEvidence } from "../lib/executionEvidence";

export function StatusChip({ status, running }: { status?: ExecStatus; running?: boolean }) {
  const evidence = executionEvidence(status, running);
  return <span className={`chip chip-${running ? "running" : status ?? "unrecorded"}`} data-tip={evidence.detail}>{evidence.label}</span>;
}
</storycode:copy>

### Assemble `apps/web/src/components/StatusChip.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/StatusChip.tsx"><storycode:paste select="#f-b0293590899e-StatusChip"/></storycode:file>

<a id="output-b528549051a2"></a>
## `apps/web/src/lib/executionEvidence.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A run result is distinct from what was checked](contract.md).

<a id="f-b528549051a2-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-b528549051a2-expression_statement">
import { describe, expect, it } from "vitest";
import { executionEvidence } from "./executionEvidence";

// Guarantee: docs/guarantees/execution/a-run-result-is-distinct-from-what-was-checked.md
describe("execution evidence", () => {
  it("does not call a successful exit verified", () => {
    const success = executionEvidence("ok");
    expect(success.label).toContain("run succeeded");
    expect(success.label).not.toContain("verified");
    expect(success.detail).toContain("reported separately");
  });
  it("distinguishes a stale recording, no recording, and a live run", () => {
    expect(executionEvidence("stale").label).toBe("Stale recording");
    expect(executionEvidence().label).toContain("Unrecorded");
    expect(executionEvidence("ok", true).detail).toContain("no verdict yet");
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/executionEvidence.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/executionEvidence.test.ts"><storycode:paste select="#f-b528549051a2-expression_statement"/></storycode:file>

<a id="output-4cfb93fd76ff"></a>
## `apps/web/src/lib/executionEvidence.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A run result is distinct from what was checked](contract.md).

<a id="f-4cfb93fd76ff-executionEvidence"></a>
### `executionEvidence`

<storycode:copy id="f-4cfb93fd76ff-executionEvidence">
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
</storycode:copy>

### Assemble `apps/web/src/lib/executionEvidence.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/executionEvidence.ts"><storycode:paste select="#f-4cfb93fd76ff-executionEvidence"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/CellPanel.tsx](../../../mechanisms/apps/web/src/components/CellPanel.tsx.md#output-c3c409b02e61)
- [apps/web/src/components/DiagramPanel.tsx](../../../mechanisms/apps/web/src/components/DiagramPanel.tsx.md#output-80582538ea4e)
- [apps/web/src/components/StatusChip.tsx](contract.md#output-b0293590899e)
- [apps/web/src/editor/CardRail.tsx](../../../mechanisms/apps/web/src/editor/CardRail.tsx.md#output-f6019c4774f8)
- [apps/web/src/editor/DocumentEditor.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.tsx.md#output-aab82870e517)
- [apps/web/src/lib/executionEvidence.ts](contract.md#output-4cfb93fd76ff)

### Tests and fixtures

- [apps/web/src/components/CellPanel.test.tsx](../../../mechanisms/apps/web/src/components/CellPanel.tsx.md#output-85e341faa089)
- [apps/web/src/components/DiagramPanel.test.tsx](../../../mechanisms/apps/web/src/components/DiagramPanel.tsx.md#output-56af562b717e)
- [apps/web/src/editor/DocumentEditor.test.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.test.tsx.md#output-71bbcfe029bb)
- [apps/web/src/lib/executionEvidence.test.ts](contract.md#output-b528549051a2)
- [apps/web/src/views/workspaceTabs.test.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-5370130a675d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
