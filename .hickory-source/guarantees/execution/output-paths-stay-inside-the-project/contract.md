<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Output Path Never Escapes The Document's Directory

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given any document, when a `<hick:file path=…>` declares a path that is
absolute or whose `..` components climb out of the document's output
directory, then no write, read, or drift comparison uses that path: the run
fails naming the offending path, why it was refused, and what a valid path
looks like.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/output-paths-stay-inside-the-project.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)
- [crates/hickory-cli/src/up/mod.rs](../../../mechanisms/crates/hickory-cli/src/up/mod.rs.md#output-73148a3ab9db)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
