<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Past Is Edited By Rebase, Above The Floor Only

For: Authors reviewing history and choosing where an edit should land.

Given the history lens open on a repository, when a card above the
publication floor (a draft) is reworded, moved earlier or later, or
dropped (`POST /api/git/reword`, `/api/git/move`, `/api/git/drop`), then
git's own interactive rebase runs over the drafts with a todo list this
product wrote — `reword`/`pick`, a swapped order, or a line left out — and
the editor git asks is answered from a file rather than typed. The result
is the same history git would have made from a terminal.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lenses/the-past-is-edited-by-rebase-above-the-floor.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
