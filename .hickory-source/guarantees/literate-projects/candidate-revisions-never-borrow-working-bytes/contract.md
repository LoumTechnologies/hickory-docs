<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Candidate revision checks never borrow working bytes

For: Maintainers reorganizing source while preserving the complete generated program.

Given an exact Git commit/tree or complete stage-zero index, when its literate
project is verified, then both Markdown and expected outputs come from the
captured revision object identities. Unstaged files and ancestor documents
cannot complete a partial candidate. Unmerged index stages and unsupported Git
entry kinds fail. Persisted CRDT input decodes the supplied immutable updates;
subsequent live edits do not change the captured tree.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/literate-projects/candidate-revisions-never-borrow-working-bytes.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
