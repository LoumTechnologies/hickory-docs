import type { ExecStatus } from "../api/types";

const LABELS: Record<ExecStatus, string> = {
  ok: "ok",
  failed: "failed",
  stale: "stale",
  "never-run": "never run",
};

export function StatusChip({ status, running }: { status?: ExecStatus; running?: boolean }) {
  if (running) {
    return <span className="chip chip-running">running…</span>;
  }
  const s = status ?? "never-run";
  return <span className={`chip chip-${s}`}>{LABELS[s]}</span>;
}
