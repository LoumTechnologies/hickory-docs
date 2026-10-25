<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A fill handle extends formulas

For: Authors editing executable documents and the files those documents produce.

An editable rendered table shows one small plus handle at the bottom-right of
its selected cell or rectangular selection, including a selection made in
reverse. While editing a cell the handle is hidden. Dragging the handle up,
down, left or right previews the extended range; releasing fills that range
as one document edit and selects the result. Escape or losing window focus
cancels without changing data. A source or format edit during the drag invalidates
its frozen seed, so release cannot overwrite that newer edit.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-fill-handle-extends-formulas.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
