<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Cell Cannot Hang A Run

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a document executed through the local or sandbox executor, when a cell
runs longer than its time limit — the cell's own `timeout="<seconds>"`
attribute, else `HICKORY_CELL_TIMEOUT` (seconds), else 120 seconds — then the
cell's process is really killed (on Unix, its whole process group, so
background children die with it) and the cell fails with an error naming the
container, the command, the limit that was hit, and both next steps (raise
`timeout=` on the cell, or set `HICKORY_CELL_TIMEOUT`). `timeout="0"`
declares one cell unbounded — allowed, but only explicitly — and a malformed
`timeout=` or `HICKORY_CELL_TIMEOUT` value fails loudly instead of silently
removing the limit.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-cell-cannot-hang-a-run.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
