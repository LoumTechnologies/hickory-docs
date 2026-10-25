<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Write Through The Merged View Reports Per Target, And Undo Reaches All Of Them

For: Engineers using local sessions or pairing their own machines.

Given a merged view over several worktrees, when an edit is made through it,
then it is routed either to **one** target ("just here") or to **every** target
opened for writing ("shared"); every target reports for itself; and a target
that was not opened for writing is never written at all.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/a-merged-view-write-reports-per-target.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
