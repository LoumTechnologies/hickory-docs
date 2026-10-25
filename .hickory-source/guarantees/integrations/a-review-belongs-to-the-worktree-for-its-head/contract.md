<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Review Belongs To The Worktree For Its Head

For: Engineers connecting repository issues, reviews and locally installed tools.

Given open GitHub pull requests or GitLab merge requests and local clones or
worktrees, when the workspace tree is shown, then a review appears beneath the
clone or worktree whose normalized repository and checked-out branch match its
current head; expanding it exposes its current state, reviews, comments, checks,
bounded logs, provider mergeability, and honestly labeled current-head conflict
analysis.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/integrations/a-review-belongs-to-the-worktree-for-its-head.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
