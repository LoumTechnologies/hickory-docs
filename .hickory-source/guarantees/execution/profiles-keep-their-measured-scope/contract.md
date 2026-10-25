<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Profiles keep their measured scope

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a captured or imported performance profile, function costs retain the
collector's units, measured interval, per-thread/profile scope and attribution.
Sampling observations are not calls. Inclusive costs overlap and are never summed
as independent costs. A frame's source line anchors a function; it is not a
measurement of that line's execution time. Missing samples do not prove zero cost.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/profiles-keep-their-measured-scope.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
