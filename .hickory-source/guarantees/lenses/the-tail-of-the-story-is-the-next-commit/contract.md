<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
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
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lenses/the-tail-of-the-story-is-the-next-commit.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
