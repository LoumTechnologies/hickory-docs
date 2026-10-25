<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/StatusChip.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A run result is distinct from what was checked](../../../../../contract.md)

Captured SHA-256: `5da30da59b3dda4ae75c1b2228608790cd15857d5d5c1c322a32636f896557f5`.

<book:entry path="apps/web/src/components/StatusChip.tsx">
import type { ExecStatus } from "../api/types";
import { executionEvidence } from "../lib/executionEvidence";

export function StatusChip({ status, running }: { status?: ExecStatus; running?: boolean }) {
  const evidence = executionEvidence(status, running);
  return <span className={`chip chip-${running ? "running" : status ?? "unrecorded"}`} data-tip={evidence.detail}>{evidence.label}</span>;
}
</book:entry>

</book:doc>
