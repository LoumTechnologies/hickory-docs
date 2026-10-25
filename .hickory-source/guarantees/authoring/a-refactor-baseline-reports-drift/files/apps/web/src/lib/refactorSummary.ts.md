<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/refactorSummary.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Refactor Baseline Reports The Moment An Output Would Move](../../../../../contract.md)

Captured SHA-256: `a1f910f101209ae847e29cbf5fea61655eb68b551068b6d7f8a35ce5776af38e`.

<book:entry path="apps/web/src/lib/refactorSummary.ts">
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
</book:entry>

</book:doc>
