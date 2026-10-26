<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Tail Of The Story Is The Next Commit

For: Authors reviewing history and choosing where an edit should land.

Given the history lens open on a repository, when the working tree holds
uncommitted changes, then the last card says so and offers a message box;
the message typed there, with Commit, stages everything and makes one
commit with it — `git add -A` and `git commit`, run as themselves. And
whether or not anything is uncommitted, the tail offers a command and an
output folder: Run and commit runs that command in a detached worktree at
HEAD, never in the working tree, and commits what it wrote under the folder
as HEAD's child, with `Hick-Recipe` and `Hick-Output: <tree> <folder>` in
the trailers; the folder's files then appear in the working tree, clean.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lenses/the-tail-of-the-story-is-the-next-commit.md.md). The paragraph ledger records their exact ranges and dispositions.

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
