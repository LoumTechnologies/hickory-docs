<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Observations read beside their captured source

For: Authors debugging generated code while navigating its document source.

Given a native DAP or browser interpreter session with captured UTF-8 source,
when its reading surface is expanded, then source excerpts and typed observations
share immutable revision references and evidence IDs. Requested breakpoints,
adapter-bound breakpoints and the actual paused statement are distinct facts.
Selecting a caller never moves the actual paused statement to that caller.
Values and explicit expression receipts identify their captured stop and frame;
invocation identity remains unknown when the provider cannot establish it.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/source-anchored-observation-reading.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
