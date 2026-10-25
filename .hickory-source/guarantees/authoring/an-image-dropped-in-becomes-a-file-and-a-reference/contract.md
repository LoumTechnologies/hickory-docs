<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Image Dropped In Becomes A File And A Reference

For: Authors editing executable documents and the files those documents produce.

Given an image dragged into an open document, or pasted from the clipboard,
when it lands then the bytes are written into an `assets/` directory beside
that document and the buffer gains an ordinary markdown reference to it —
`![a chart](assets/a-chart.png)` — at the point of the drop. The picture is
drawn under the line that references it. Nothing is embedded in the document,
nothing is uploaded anywhere, and the woven markdown needs no special support
for either, because there is nothing special in the document to support.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/an-image-dropped-in-becomes-a-file-and-a-reference.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
