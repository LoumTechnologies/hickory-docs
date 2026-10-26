<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Syntax Error Names The Invariant It Hit

For: Authors editing executable documents and the files those documents produce.

Given a document whose prose quotes hick's own tag syntax — a sentence or
code span containing literal text shaped like `<prefix:name attr="value">` —
when that text is malformed enough to fail parsing, then the error explains
that hick has no escaping and points at the `h:`-prefix convention for
writing about hick's own syntax, rather than naming only the unexpected
token.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-syntax-error-names-the-invariant-it-hit.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hick-lang/src/lib.rs](../../../mechanisms/crates/hick-lang/src/lib.rs.md#output-0cf7120750a0)

### Tests and fixtures

- [crates/hick-lang/src/lib/tests/tests.rs](../../../mechanisms/crates/hick-lang/src/lib/tests/tests.rs.md#output-e91f067d75eb)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
