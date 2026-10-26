<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Rewritten Link Keeps Its Paragraph Attached

For: Authors editing generated text and reviewers tracing a result to its source.

Given a paragraph of prose in a `.md` document that a transform cuts into
pieces — a `hick:val` substitution, a derived passage — when the document is
woven, then every piece that came through UNCHANGED still carries the source
bytes it came from, narrowed to exactly its own extent; only the bytes a
transform actually replaced are reported as synthetic. A ribbon is drawn from
the sentence in the source to the sentence in the weave, and an edit made on
the woven side still maps back.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/a-rewritten-link-keeps-its-paragraph-attached.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hick-flow/src/transform.rs](../../../mechanisms/crates/hick-flow/src/transform.rs.md#output-88934a253aef)
- [crates/hick-literate/src/links.rs](../../../mechanisms/crates/hick-literate/src/links.rs.md#output-95b0a93afc31)
- [crates/hickory-lineage/src/lib.rs](../../../mechanisms/crates/hickory-lineage/src/lib.rs.md#output-bc43b0352ff3)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
