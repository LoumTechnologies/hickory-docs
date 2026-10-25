<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Visit Is Reported Exactly Once Per Page Load

For: Maintainers evaluating landing-page behavior and visitor privacy.

Given the homepage, when it mounts, then exactly one
`landing_viewed` event fires for that page load, regardless of how many times
React re-renders or re-mounts the view. In particular, React's StrictMode
double-mount in development must not produce two events.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/analytics/landing-events-fire-once.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
