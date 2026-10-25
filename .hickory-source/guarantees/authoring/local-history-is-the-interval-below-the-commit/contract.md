<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Local History Is The Interval Below The Commit

For: Authors editing executable documents and the files those documents produce.

Given a folder of documents, when anything writes to it — you, a run, a weave,
a reverse edit, an ingest, the agent, find-and-replace, the merge driver —
then what the affected files held immediately before is kept, locally, and can
be put back:

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/local-history-is-the-interval-below-the-commit.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
