<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Ingest Does Not Require The Rest Of The Document To Already Pass

For: Authors editing executable documents and the files those documents produce.

Given a document whose target cell (the one named by `hick ingest --from
'#id'`) does not depend on some OTHER cell elsewhere in the document, when
that other cell currently fails, then the ingest still succeeds — it runs
only the target cell and its real dependencies, never the whole document.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/ingest-does-not-require-the-rest-of-the-document-to-already-pass.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
