<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Landing Attribution Credits The First Touch, And Survives The Hash Router

For: Maintainers evaluating landing-page behavior and visitor privacy.

Given a visitor arriving with UTM parameters in the query string
(`https://…/?utm_source=hn&utm_campaign=launch#/`), when the landing page
reports any event, then those parameters are read from the query string —
which sits before the fragment, so the hash router never consumes them — and
the *first* tagged arrival is recorded permanently as that visitor's first
touch. A later visit from a different source reports its own `utm_*` values
but does not overwrite the first touch or the derived `intended_segment`. An
untagged arrival never stamps a first touch, so opening the bare domain
before clicking an ad does not permanently credit the visitor as direct.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/analytics/landing-attribution-is-first-touch.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
