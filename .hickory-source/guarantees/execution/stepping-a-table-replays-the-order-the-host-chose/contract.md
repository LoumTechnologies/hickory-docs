<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Stepping A Table Replays The Order The Host Chose

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a table whose formulas can be evaluated, when the author steps through
them, then they see every formula cell's turn in the order it actually ran —
what the cell read, what those references were worth at that moment, which
batch it went out in, and what it came to — and those steps are the same
evaluation the grid is showing, not a second one.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/stepping-a-table-replays-the-order-the-host-chose.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
