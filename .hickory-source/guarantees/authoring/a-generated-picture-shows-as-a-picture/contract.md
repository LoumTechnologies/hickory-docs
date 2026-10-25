<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Generated Picture Shows As A Picture

For: Authors editing executable documents and the files those documents produce.

Given a `<hick:file>` block whose `path` names something a browser can draw
(`.svg`, `.png`, `.jpg`/`.jpeg`, `.gif`, `.webp`, `.avif`), when the document
is opened in the app, then the block shows **the picture the file holds**, and
a `▨` icon on the action rail toggles it back to the source that draws it —
the plotting code, the cell, and the tags around them.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-generated-picture-shows-as-a-picture.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
