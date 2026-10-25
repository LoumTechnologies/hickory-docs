<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Earlier Cell's Output Survives A Later Cell's Failure

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a `hick run` where a cell mounting an output volume genuinely
succeeds, and a LATER cell in the same run then fails, then that earlier
cell's real output is inspectable on disk under `.hick-cache/last-run/` —
never silently gone just because the run, as a whole, did not finish.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/an-earlier-cells-output-survives-a-later-cells-failure.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
