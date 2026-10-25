<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Driving The Home-Page Demo Is Reported As Revealed Interest

For: Maintainers evaluating landing-page behavior and visitor privacy.

Given a visitor on the home page, when they run the executable cells, edit the
document, or edit a generated file, then a `demo_engaged` event is sent
carrying `demo_id` (which demo) and `step` (what they did), alongside the
attribution and declared-segment properties every landing event carries. It is
sent for the **first** interaction only, so the count means "this visitor tried
the product" rather than "this visitor typed n characters".

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/analytics/landing-demo-engagement.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
