<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Drift Report Names What Differs

For: Reviewers deciding what an output, recording or prediction actually establishes.

Given `hick test` reports a committed output file as drifted, then the
message names where the mismatch starts — a byte offset, and a line number
when the content is text — and how the two lengths compare, not only the
fact that the two differ.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/a-drift-report-names-what-differs.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
