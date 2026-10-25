<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Three Provenances Are Drawn Apart, And Any Subset Can Be Shown

For: Authors editing generated text and reviewers tracing a result to its source.

Given a document on screen, when the overlay draws where its text came from,
then it draws up to three families — **lineage** (the weave's byte-exact
derivation), **context** (what was in front of the model when an agent wrote
these lines, from the session record), and **declared** (what the author says
it cites, `cites="…"`) — each in its own hue AND its own stroke (solid
palette / dotted amber / dashed grey), each with a title that says what it is
and what it cannot claim; and three toggles in the status bar show any
subset, remembered per browser.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/three-provenances-are-drawn-apart.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
