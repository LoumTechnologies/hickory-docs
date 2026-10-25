<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Report Is Not a Reproducible Artifact

For: Reviewers deciding what an output, recording or prediction actually establishes.

Given a document whose root declares `volatile="true"`, when `hick test`
runs, then its woven output is excluded from drift comparison while every
other output of that document is still compared byte-for-byte, and every
`hick:expect` expectation is still enforced.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/a-report-is-not-a-reproducible-artifact.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
