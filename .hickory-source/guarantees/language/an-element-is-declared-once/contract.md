<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Element Is Declared Once

For: Authors embedding code in Markdown and maintainers changing the document grammar.

Given a tag the app draws — `exec`, `file`, `diagram`, the prose between
them, and every element added after this — when the block model is built
for a document, then the tag's name, the attributes it accepts, how it
renders to a block, which of its children the walk visits, and the actions
it answers are all read from one declaration: an implementation of
`hick_blocks::Element`, registered once in one `Registry`. The block on
the wire is `{kind, span, …props}` for every element alike, `kind` naming
the component that draws it, `span` the tag's byte span; and the registry
can describe its whole vocabulary as data.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/language/an-element-is-declared-once.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hick-blocks/src/lib.rs](../../../mechanisms/crates/hick-blocks/src/lib.rs.md#output-5260198b0a7b)
- [crates/hick-literate/src/render.rs](../../../mechanisms/crates/hick-literate/src/render.rs.md#output-4a7f39f68eb8)
- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)

### Tests and fixtures

- [crates/hickory-cli/tests/block_actions.rs](../../../mechanisms/crates/hickory-cli/tests/block_actions.rs.md#output-fa0f3e293d18)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
