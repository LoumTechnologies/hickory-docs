<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Denied Mach Lookup Explains Itself

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given `HICKORY_EXECUTOR=sandbox` on macOS, when a cell's command is refused
because the Seatbelt `Profile::Cell` policy grants no `(allow mach-lookup)`,
then the error a person sees names the sandbox and the fix
(`HICKORY_EXECUTOR=local`), not only the raw string the denied system service
returned.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-denied-mach-lookup-explains-itself.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hickory-executor-sandbox/src/lib.rs](../../../mechanisms/crates/hickory-executor-sandbox/src/lib.rs.md#output-3ed8f409da24)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
