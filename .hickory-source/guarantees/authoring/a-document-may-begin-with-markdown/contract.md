<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Document May Begin With Markdown

For: Authors editing executable documents and the files those documents produce.

Given a `.md` file whose prologue does not open a `<PREFIX:doc>` element,
when it is parsed, then the whole file is the body of an implicit root: there
is no wrapper to write at the top and none to close at the bottom, the
namespace prefix is `hick` without any `xmlns:` declaration, and every byte of
prose — before, between, and after any tags — survives byte-for-byte. When the
file *does* open that element in its prologue, it is parsed exactly as it
always was.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-document-may-begin-with-markdown.md.md). The paragraph ledger records their exact ranges and dispositions.

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
