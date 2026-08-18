// Words for the refactor badge, as arithmetic on the status the server
// answers — pure, so the truth-telling is testable without a component.

import type { RefactorStatus } from "../api/types";

/** The badge's text, or null while no baseline is pinned. */
export function refactorSummary(status: RefactorStatus | null): string | null {
  if (!status || !status.active) return null;
  if (status.clean) return "Outputs match baseline";
  const n = status.diffs.length;
  return n === 1 ? "1 output differs" : `${n} outputs differ`;
}

/** The hover detail: which outputs moved, and how. Empty string when clean. */
export function refactorDetail(status: RefactorStatus | null): string {
  if (!status || !status.active || status.clean) return "";
  return status.diffs.map((d) => `${d.path} (${d.kind})`).join(", ");
}
