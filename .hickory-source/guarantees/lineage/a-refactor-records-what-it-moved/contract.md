<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Refactor Records What It Moved — Byte-Exactly, And Only When Continuity Is On

For: Authors editing generated text and reviewers tracing a result to its source.

Given a pinned refactor baseline, when the baseline is ended and the
restructured document still produces byte-identical outputs, then every span
that moved is recorded in the correspondence journal as a **byte-precise**
fact; and when continuity is off for the project, then nothing is recorded
anywhere.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/a-refactor-records-what-it-moved.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
