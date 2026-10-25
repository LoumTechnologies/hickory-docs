<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Breakpoint That Is Not Bound Yet Is Not Refused

For: Authors debugging generated code while navigating its document source.

Given a breakpoint set on a document line, when the debugger reports on it,
then it has **three** states rather than two, because "the adapter has not
confirmed this" and "there is nothing here to stop on" are different facts and
a person acts differently on each:

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/a-breakpoint-that-is-not-bound-yet-is-not-refused.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
