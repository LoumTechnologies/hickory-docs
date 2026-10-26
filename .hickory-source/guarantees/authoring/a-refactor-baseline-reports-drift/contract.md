<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Refactor Baseline Reports The Moment An Output Would Move

For: Authors editing executable documents and the files those documents produce.

Given a document whose woven outputs were pinned as a refactor baseline
(the toolbar's "Refactor", `POST /api/docs/:id/refactor/begin`), when the
document is restructured — blocks split, prose interleaved, sections
reordered — then the live verdict (`GET …/refactor/status`, the toolbar
badge) says either that every output still matches the baseline or exactly
which outputs differ and how (changed, added, removed); and checking never
executes a cell.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-refactor-baseline-reports-drift.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-6b96c08c5a65"></a>
## `apps/web/src/lib/refactorSummary.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Refactor Baseline Reports The Moment An Output Would Move](contract.md).

<a id="f-6b96c08c5a65-expression_statement"></a>
### `expression_statement`

The refactor badge's words. Protects the reporting half of

<storycode:copy id="f-6b96c08c5a65-expression_statement">
// The refactor badge's words. Protects the reporting half of
// docs/guarantees/authoring/a-refactor-baseline-reports-drift.md.

import { describe, expect, it } from "vitest";

import { refactorDetail, refactorSummary } from "./refactorSummary";

describe("refactorSummary", () => {
  it("says nothing while no baseline is pinned", () => {
    expect(refactorSummary(null)).toBeNull();
    expect(refactorSummary({ active: false })).toBeNull();
  });

  it("reports a clean baseline and counts the outputs that moved", () => {
    expect(
      refactorSummary({ active: true, started_at: "t", clean: true, diffs: [] }),
    ).toBe("Outputs match baseline");
    expect(
      refactorSummary({
        active: true,
        started_at: "t",
        clean: false,
        diffs: [{ path: "a.py", kind: "changed" }],
      }),
    ).toBe("1 output differs");
    expect(
      refactorSummary({
        active: true,
        started_at: "t",
        clean: false,
        diffs: [
          { path: "a.py", kind: "changed" },
          { path: "b.txt", kind: "removed" },
        ],
      }),
    ).toBe("2 outputs differ");
  });

  it("details name each output and how it moved", () => {
    expect(refactorDetail({ active: true, started_at: "t", clean: true, diffs: [] })).toBe("");
    expect(
      refactorDetail({
        active: true,
        started_at: "t",
        clean: false,
        diffs: [
          { path: "a.py", kind: "changed" },
          { path: "new.txt", kind: "added" },
        ],
      }),
    ).toBe("a.py (changed), new.txt (added)");
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/refactorSummary.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/refactorSummary.test.ts"><storycode:paste select="#f-6b96c08c5a65-expression_statement"/></storycode:file>

<a id="output-4c2fc13d2e5d"></a>
## `apps/web/src/lib/refactorSummary.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Refactor Baseline Reports The Moment An Output Would Move](contract.md).

<a id="f-4c2fc13d2e5d-refactorSummary"></a>
### `refactorSummary`

Words for the refactor badge, as arithmetic on the status the server
answers — pure, so the truth-telling is testable without a component.

<storycode:copy id="f-4c2fc13d2e5d-refactorSummary">
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

</storycode:copy>
<a id="f-4c2fc13d2e5d-refactorDetail"></a>
### `refactorDetail`

The hover detail: which outputs moved, and how. Empty string when clean.

<storycode:copy id="f-4c2fc13d2e5d-refactorDetail">
/** The hover detail: which outputs moved, and how. Empty string when clean. */
export function refactorDetail(status: RefactorStatus | null): string {
  if (!status || !status.active || status.clean) return "";
  return status.diffs.map((d) => `${d.path} (${d.kind})`).join(", ");
}
</storycode:copy>

### Assemble `apps/web/src/lib/refactorSummary.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/refactorSummary.ts"><storycode:paste select="#f-4c2fc13d2e5d-refactorSummary"/><storycode:paste select="#f-4c2fc13d2e5d-refactorDetail"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/RefactorBadge.tsx](../../../mechanisms/apps/web/src/components/RefactorBadge.tsx.md#output-c40aef0d3c52)
- [apps/web/src/lib/refactorSummary.ts](contract.md#output-4c2fc13d2e5d)
- [apps/web/src/views/workspaceTabs.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-b5a48fd337ae)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)
- [crates/hickory-cli/src/serve/refactor.rs](../../../mechanisms/crates/hickory-cli/src/serve/refactor.rs.md#output-3a011c557915)

### Tests and fixtures

- [apps/web/src/lib/refactorSummary.test.ts](contract.md#output-6b96c08c5a65)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
