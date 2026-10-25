<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Recipe Commit Can Be Replayed, Into A New Commit, Never Over The Old One

For: Authors reviewing history and choosing where an edit should land.

Given a commit whose trailers carry a recipe and whose tree matches its
recorded output, when Replay is pressed on its card in the history lens
(`POST /api/git/replay {sha}`), then the recipe's command is run again in a
detached worktree — never in the person's working tree — and what it wrote
is committed as a **new** recipe commit S2 carrying `Hick-Replay-Of` (the
commit replayed) and `Hick-Replay-Same` (whether the output tree is the one
that commit recorded); then S2 is joined to the history:

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lenses/a-recipe-commit-can-be-replayed.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
