<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Unwatched Edit Leaves Its Correspondence Behind Before It Is Committed

For: Authors editing generated text and reviewers tracing a result to its source.

Given continuity is on for a project, when a `.md` document is staged whose
edit no recording site saw, then the pre-commit hook reconstructs what moved
and records it at **line** precision — and the commit is never blocked.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/an-unwatched-edit-is-repaired-not-refused.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
