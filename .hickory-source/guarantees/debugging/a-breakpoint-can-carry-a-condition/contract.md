<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Breakpoint Can Carry A Condition

For: Authors debugging generated code while navigating its document source.

Given a breakpoint in a document or a plain file, when it is right-clicked or
alt-clicked in the gutter, then a panel opens on that line offering three
things — a **condition**, a **hit count**, and a **log message** — and what is
typed reaches the adapter on the next `setBreakpoints`, or on the next run if
nothing is running.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/a-breakpoint-can-carry-a-condition.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
