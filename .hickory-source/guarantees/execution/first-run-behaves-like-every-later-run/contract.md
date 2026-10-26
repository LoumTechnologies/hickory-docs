<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Document's First Run Behaves Like Every Later Run

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a document that assembles a file with `hick:file` and then executes that
file from a volume, when it is run for the FIRST time — with nothing previously
committed — then the cell finds the file and the run succeeds, exactly as it
would on any later run.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/first-run-behaves-like-every-later-run.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve; no-existing-code-or-test-reference-resolved.

</storycode:doc>
