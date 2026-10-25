<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Every Hover Hint Is Drawn In The App Theme

For: Authors editing executable documents and the files those documents produce.

Given any control, row, or gutter marker in the app that explains itself on
hover, when the pointer rests on it (or keyboard focus reaches it), then the
explanation is drawn by the app in the palette the user chose — the same
tokens as every other raised surface — and never by the operating system's
`title=` tooltip.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/every-hover-hint-is-drawn-in-the-app-theme.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
