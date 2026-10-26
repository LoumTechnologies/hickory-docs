<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Drift Report Names What Differs

For: Reviewers deciding what an output, recording or prediction actually establishes.

Given `hick test` reports a committed output file as drifted, then the
message names where the mismatch starts — a byte offset, and a line number
when the content is text — and how the two lengths compare, not only the
fact that the two differ.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/a-drift-report-names-what-differs.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)

### Tests and fixtures

- [crates/hickory-cli/src/lib_tests.rs](../../../mechanisms/crates/hickory-cli/src/lib_tests.rs.md#output-47cc90118acd)
- [crates/hickory-cli/tests/volatile_outputs.rs](../a-report-is-not-a-reproducible-artifact/contract.md#output-c6bfdff8fc14)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
