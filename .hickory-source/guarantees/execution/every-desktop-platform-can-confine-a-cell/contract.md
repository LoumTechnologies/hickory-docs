<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Every Desktop Platform Can Confine A Cell

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given `HICKORY_EXECUTOR=sandbox` on Linux, macOS **or Windows**, when a
document's cell runs, then it may write only its own workdir, reaches the
network only if the document declared it, and cannot outlive the run that
started it. Where nothing can enforce that, the executor **refuses to run**
rather than running the cell unconfined.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/every-desktop-platform-can-confine-a-cell.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
