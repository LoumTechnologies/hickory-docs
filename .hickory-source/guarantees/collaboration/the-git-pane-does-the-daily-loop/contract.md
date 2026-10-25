<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Git Pane Does The Daily Loop

For: Engineers using local sessions or pairing their own machines.

Given a folder that is a git repository, when the Git pane is open, then
above the commit graph it shows the working tree — the branch, its upstream
and how far apart they are, every changed file on the staged and unstaged
sides — and offers, each as one git command run as itself: stage and unstage
a file or everything, discard a file's changes after a second click, show a
file's diff, commit what is staged with a message, amend the last commit,
push (setting an upstream the first time), pull fast-forward only, switch to
or create a branch, and stash and pop. When git refuses, git's own words are
shown. Amending is refused when `HEAD` is below the publication floor, since
that commit is a record somebody else may hold. Nothing prompts for a
credential; a push that needs one fails saying so rather than hanging.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/the-git-pane-does-the-daily-loop.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
