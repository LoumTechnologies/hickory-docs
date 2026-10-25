<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Element Is Drawn By Its View

For: Authors embedding code in Markdown and maintainers changing the document grammar.

Given a block the editor renders in place of its source — a cell, a
diagram, a formula, a table, a picture — when the Document view mounts
something in the block's place, then what it mounts is the one view
registered for that block's `kind` in `apps/web/src/elements`, and which
blocks are renderable at all is what those views say they draw. The
editor itself has no branch per element: it asks the registry which view
draws a block, folds the block's lines, and hands the view its slot and
one context object. A cell's Run goes through the element's `run` action
(`POST /api/docs/:id/blocks/:at/:action`) when the render has given the
cell a span to address, and through the older run route until it has.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/language/an-element-is-drawn-by-its-view.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
