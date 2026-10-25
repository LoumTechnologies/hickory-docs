<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Lineage Browser Shows One Column Per Stage

For: Authors editing executable documents and the files those documents produce.

Given a project of documents connected into a pipeline, when the lineage
browser opens it, then there is one column per **stage** — a document plus the
files it generates — named for the stage, with one of the stage's files opened
in place inside the stage's own file tree; and every link drawn between blocks
is one the weaver computed, never one the view invented.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-lineage-browser-shows-one-column-per-stage.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
