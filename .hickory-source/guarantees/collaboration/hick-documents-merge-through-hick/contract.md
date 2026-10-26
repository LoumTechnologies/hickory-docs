<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# `.md` Documents Merge Through Hick, And A Clone That Cannot Is Told

For: Engineers using local sessions or pairing their own machines.

Given a repository set up with `hick init`, when git merges a `.md`
document, then it runs hick's merge driver rather than its own line merge —
and when a clone has the routing but not the driver definition, then that is
reported at project open and by `hick test`, because git otherwise falls back
silently.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/hick-documents-merge-through-hick.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/components/MergeDriverNotice.tsx](../../../mechanisms/apps/web/src/components/MergeDriverNotice.tsx.md#output-c97d597c05fd)
- [crates/hickory-cli/src/init.rs](../../../mechanisms/crates/hickory-cli/src/init.rs.md#output-a329ec22d0b8)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)
- [crates/hickory-cli/src/merge_driver.rs](../../../mechanisms/crates/hickory-cli/src/merge_driver.rs.md#output-75494bcb5e9d)
- [crates/hickory-cli/src/serve/history.rs](../../../mechanisms/crates/hickory-cli/src/serve/history.rs.md#output-be949b7ea526)

### Tests and fixtures

- [apps/web/src/components/MergeDriverNotice.test.tsx](../../../mechanisms/apps/web/src/components/MergeDriverNotice.tsx.md#output-38c5ddfe6997)
- [crates/hickory-cli/tests/replay_and_floor.rs](../../../mechanisms/crates/hickory-cli/tests/replay_and_floor.rs.md#output-652493fd98fd)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
