<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Several Cursors Edit At Once

For: Authors editing executable documents and the files those documents produce.

Given any editor in the app — a document, a plain file, a generated file —
when the person Alt+clicks, Alt+drags a column, or presses Ctrl+D (Cmd+D) on
a selection, then a second cursor or selection is added rather than the first
moved, Ctrl+Shift+L selects every occurrence, and typing, deleting and
pasting act at every cursor at once.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/several-cursors-edit-at-once.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
