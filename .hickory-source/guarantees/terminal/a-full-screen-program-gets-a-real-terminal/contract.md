<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Full-Screen Program Gets A Real Terminal

For: Authors running interactive tools alongside documents without losing session state.

Given a session running a program that takes over the screen — an editor, a
process monitor, a pager, another vendor's coding agent — when it runs in one
of these terminals, then it gets everything it needs to draw itself:

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/terminal/a-full-screen-program-gets-a-real-terminal.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
