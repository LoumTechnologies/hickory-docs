<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Watched Cell Shows What The Terminal Showed

For: Authors running interactive tools alongside documents without losing session state.

Given a `hick:exec` cell that has run — or is running now, with its
transcript arriving on the run channel — when its output appears under the
cell in the Document view, then it is drawn by a terminal emulator rather
than printed as text, so what the reader sees is what the command actually
drew:

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/terminal/a-watched-cell-shows-what-the-terminal-showed.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
