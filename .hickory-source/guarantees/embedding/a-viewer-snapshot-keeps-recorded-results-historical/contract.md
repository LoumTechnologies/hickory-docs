<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A viewer snapshot keeps recorded results historical

For: Host developers embedding the document editor without a native execution service.

Given a document that can be woven by the local application, exporting its viewer
snapshot captures source bytes, their SHA-256, capture time and the cell model
with recorded results. Export does not execute checks. The standalone browser
viewer uses the existing document embed and CellPanel to display that source,
stdout/stderr transcripts, statuses and expectation differences without a local
backend. It labels the results historical and does not certify current files.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/embedding/a-viewer-snapshot-keeps-recorded-results-historical.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
