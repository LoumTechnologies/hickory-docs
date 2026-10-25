<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Merged View Is A Lens Over Several Worktrees, Never A Document

For: Engineers using local sessions or pairing their own machines.

Given a file that exists in several worktrees of one repository, when a merged
view is opened over it, then regions **every** source agrees on appear once,
regions that differ appear as variants naming each source, and the view says
plainly that it is read-only by default and exists on disk nowhere. Explicit
region edits name the targets opened for writing and use the captured source
preconditions; they do not give the lens a save path.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/the-merged-view-is-a-lens.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
