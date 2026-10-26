<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Silent Tag Does Not Double A Blank Line

For: Authors editing executable documents and the files those documents produce.

Given a `<hick:copy>`, `<hick:cut>`, `<hick:container>`, `<hick:volume>`,
`<hick:feature>`, `<hick:needs>`, or `<hick:allow>` tag standing alone
between two paragraphs, when the document is woven, then exactly one blank
line separates the paragraphs — never the three or more a naive
concatenation of the surrounding prose's own paragraph-break newlines would
leave behind.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-silent-tag-does-not-double-a-blank-line.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hick-handlers/src/handlers/copy.rs](../../../mechanisms/crates/hick-handlers/src/handlers/copy.rs.md#output-2bc5aa33e241)
- [crates/hick-literate/src/weave.rs](../../../mechanisms/crates/hick-literate/src/weave.rs.md#output-7fd582622cb2)

### Tests and fixtures

- [crates/hickory-cli/tests/weave_reads_naturally.rs](../../../mechanisms/crates/hickory-cli/tests/weave_reads_naturally.rs.md#output-a118c221b58a)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
