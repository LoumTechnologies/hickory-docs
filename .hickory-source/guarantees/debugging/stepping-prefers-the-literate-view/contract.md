<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Stepping prefers the launching literate view

For: Authors debugging generated code while navigating its document source.

Given a debugging session launched from a literate view, when it pauses at a
line represented by that view, then navigation reveals the line in the view
instead of opening its ordinary source file. This also applies to a callee in
another represented file. The paused-line highlight follows the mapped frame.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/stepping-prefers-the-literate-view.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
