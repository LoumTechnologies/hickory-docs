<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Cell With An Id Is Quotable, And The Quote Keeps The Cell's Provenance

For: Authors editing executable documents and the files those documents produce.

Given `<hick:exec id="p95" …>` that shows output, when another place in the
document says `<hick:paste select="#p95" />` — before or after the cell, in
prose or in a generated file — then what the cell SHOWS (its transcript,
rendered per `show=`, without the trailing line break) is pasted there, and
the pasted bytes carry the cell's **exec** origin, so `hick lineage` and the
app's ribbons end at the computation rather than one hop short of it.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-cell-with-an-id-is-quotable.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
