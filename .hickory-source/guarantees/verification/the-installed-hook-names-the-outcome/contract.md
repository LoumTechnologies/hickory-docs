<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Pre-Commit Hook `hick init` Installs Names Which Outcome Blocked The Commit

For: Reviewers deciding what an output, recording or prediction actually establishes.

Given a repository set up by `hick init`, when the installed pre-commit
hook blocks a commit because `hick test` exited non-zero, then the message
names the specific outcome that occurred — drift (exit `1`), not verified
(exit `2`), or a failed expectation (exit `3`) — and states the fix for that
outcome and only that outcome.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/the-installed-hook-names-the-outcome.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
