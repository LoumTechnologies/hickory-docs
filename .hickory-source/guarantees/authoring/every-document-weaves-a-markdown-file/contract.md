<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Every Document Weaves A Markdown File Of Its Own Name

For: Authors editing executable documents and the files those documents produce.

Given a `.md` document loaded from a path, when it is run or woven, then it
writes a markdown file of its own name beside it — `notes/standup.md` weaves
`notes/standup.md` — unless it names a different one with `weave=`, or opts out
entirely with `weave="none"`.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/every-document-weaves-a-markdown-file.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hick-lang/src/lib.rs](../../../mechanisms/crates/hick-lang/src/lib.rs.md#output-0cf7120750a0)
- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)
- [crates/hick-literate/src/weave.rs](../../../mechanisms/crates/hick-literate/src/weave.rs.md#output-7fd582622cb2)
- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)

### Tests and fixtures

- [crates/hick-literate/tests/pipeline_tests.rs](../../../mechanisms/crates/hick-literate/tests/pipeline_tests.rs.md#output-a5d24a6d8973)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
