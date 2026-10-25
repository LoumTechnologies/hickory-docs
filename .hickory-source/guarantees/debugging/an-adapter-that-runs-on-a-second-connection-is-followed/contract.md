<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Adapter That Runs On A Second Connection Is Followed

For: Authors debugging generated code while navigating its document source.

Given an adapter that launches a supervisor rather than a program — js-debug
today — when a session starts, then hick answers its `startDebugging` request,
opens a **sibling connection to the same server**, and runs the real session
there: breakpoints, stack, values and stepping all belong to the child. The
supervising connection is held open for the life of the session, because it
owns the server process, and is shut down after the child.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/an-adapter-that-runs-on-a-second-connection-is-followed.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
