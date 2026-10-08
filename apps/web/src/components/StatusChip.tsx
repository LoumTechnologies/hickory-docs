import type { ExecStatus } from "../api/types";
import { executionEvidence } from "../lib/executionEvidence";

export function StatusChip({ status, running }: { status?: ExecStatus; running?: boolean }) {
  const evidence = executionEvidence(status, running);
  return <span className={`chip chip-${running ? "running" : status ?? "unrecorded"}`} data-tip={evidence.detail}>{evidence.label}</span>;
}
