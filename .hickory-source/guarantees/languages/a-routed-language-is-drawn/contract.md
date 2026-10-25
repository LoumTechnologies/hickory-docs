<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Routed Language Is Drawn, Or Says It Is Not

For: Teams choosing a toolchain and writing generators in their existing language.

Given a language Hickory routes, when a file or a `hick:file` block holding
it is opened — in the document editor, the output pane, or the plain-file
pane — then it is syntax-highlighted, in exactly the set of languages
`hick lang`'s `DRAW` column reports as `yes`.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/languages/a-routed-language-is-drawn.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
