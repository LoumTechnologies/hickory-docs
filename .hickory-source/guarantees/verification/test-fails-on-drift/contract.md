<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# `hick test` Fails When Documented Output Drifts From Real Output

For: Reviewers deciding what an output, recording or prediction actually establishes.

Given a document containing `<hick:expect>` blocks, when `hick test`
re-executes the document and any produced output no longer satisfies its
expectation (`exact` byte equality, or `regex-lines` full-line regex match),
then the command exits non-zero and reports the failing block with its source
span — documentation drift is a build failure, never a warning.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/test-fails-on-drift.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
