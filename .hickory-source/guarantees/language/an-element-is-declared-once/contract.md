<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
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
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/language/an-element-is-declared-once.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
