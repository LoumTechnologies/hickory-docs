<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
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


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lenses/a-recipe-commit-can-be-replayed.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/views/HistoryLens.tsx](../../../mechanisms/apps/web/src/views/HistoryLens.tsx.md#output-b01a7a57f285)
- [crates/hickory-cli/src/recipe.rs](../../../mechanisms/crates/hickory-cli/src/recipe.rs.md#output-46a50c4e20ec)
- [crates/hickory-cli/src/serve/git.rs](../../../mechanisms/crates/hickory-cli/src/serve/git.rs.md#output-3794196055eb)
- [crates/hickory-cli/src/serve/story.rs](../../../mechanisms/crates/hickory-cli/src/serve/story.rs.md#output-8d8f5369ceb4)

### Tests and fixtures

- [apps/web/src/views/HistoryLens.test.tsx](../../../mechanisms/apps/web/src/views/HistoryLens.tsx.md#output-8206e3ef4d86)
- [crates/hickory-cli/tests/story.rs](../../../mechanisms/crates/hickory-cli/tests/story.rs.md#output-5a8b52a4b73b)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
