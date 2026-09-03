import type { ExecStatus } from "../api/types";

const LABELS: Record<ExecStatus, string> = {
  ok: "ok",
  failed: "failed",
  stale: "stale",
  "unrecorded": "never run",
};

export function StatusChip({ status, running }: { status?: ExecStatus; running?: boolean }) {
  if (running) {
    return <span className="chip chip-running">running…</span>;
  }
  const s = status ?? "unrecorded";
  return <span className={`chip chip-${s}`}>{LABELS[s]}</span>;
}
