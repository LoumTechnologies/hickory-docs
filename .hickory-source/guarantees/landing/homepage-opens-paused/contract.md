<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The homepage opens with a live, paused debugger

For: Prospective users checking what the homepage demonstrates and promises.

Given a developer visiting the homepage, when its static assets load, then
one short Markdown document opens with a real browser debug session paused
at its return statement. The breakpoint, selected frame, highlighted line
and variables come from executing that document's code.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/landing/homepage-opens-paused.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
