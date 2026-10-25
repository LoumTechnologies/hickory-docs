<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Publication Floor Is Computed On Every Read And Shown

For: Engineers using local sessions or pairing their own machines.

Given a git repository open in the app, when the commit graph is read, then
the **publication floor** — `merge-base(HEAD, <published ref>)` — is computed
and reported alongside the commits, and every commit above it is marked a
draft with a sentence saying why.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/the-publication-floor-is-computed-and-shown.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
