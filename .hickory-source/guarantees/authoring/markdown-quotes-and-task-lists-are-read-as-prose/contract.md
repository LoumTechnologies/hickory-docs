<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Markdown Quotes And Task Lists Are Read As Prose

For: Authors editing executable documents and the files those documents produce.

Given any buffer this app styles as markdown — a `.md` document's prose, a
woven `.md` output pane, a plain markdown file — when a line opens with one or
more `>` markers, then that line is drawn as a quote at its nesting depth; and
when a list item's marker is followed by `[ ]` or `[x]`, then its box is drawn
as a checkbox that a single click toggles in the source.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/markdown-quotes-and-task-lists-are-read-as-prose.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
