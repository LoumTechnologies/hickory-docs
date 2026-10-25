<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Browser debugging runs edited source

For: Authors debugging generated code while navigating its document source.

Given a document containing a literal JavaScript or TypeScript file using the
advertised ES5 function/statement subset,
when the visitor explicitly starts Debug,
then a worker executes that frozen source with a real step-capable interpreter.
Breakpoints stop before executable statements, stepping can enter and leave a
user function, and inspected variables come from live scopes. Editing and
restarting changes the result. A source edit marks a live session as belonging
to the previous revision and removes its marker from the new bytes.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/browser-debugging-runs-edited-source.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
