<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Write Through The Merged View Reports Per Target, And Undo Reaches All Of Them

For: Engineers using local sessions or pairing their own machines.

Given a merged view over several worktrees, when an edit is made through it,
then it is routed either to **one** target ("just here") or to **every** target
opened for writing ("shared"); every target reports for itself; and a target
that was not opened for writing is never written at all.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/a-merged-view-write-reports-per-target.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hick-merge/src/nway.rs](../../../mechanisms/crates/hick-merge/src/nway.rs.md#output-fe8bfa541f0f)
- [crates/hickory-cli/src/serve/merged.rs](../../../mechanisms/crates/hickory-cli/src/serve/merged.rs.md#output-772c9c304ee4)
- [crates/hickory-cli/src/serve/merged_write.rs](../../../mechanisms/crates/hickory-cli/src/serve/merged_write.rs.md#output-1635aa80d82b)

### Tests and fixtures

- [crates/hickory-cli/tests/merged_view.rs](../../../mechanisms/crates/hickory-cli/tests/merged_view.rs.md#output-87756f3e699e)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
