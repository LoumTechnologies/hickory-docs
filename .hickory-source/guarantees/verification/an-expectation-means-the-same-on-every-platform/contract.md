<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Expectation Means The Same Thing On Every Platform

For: Reviewers deciding what an output, recording or prediction actually establishes.

Given a document whose `<hick:exec>` cell carries an `<hick:expect>`, when the
document is run on any supported platform, then the cell's captured output is
recorded with `\n` line endings — a `\r\n` emitted by the shell, the program,
or the container becomes `\n` at capture, before it reaches the transcript
entry, the returned stdout, the woven `.md`, or a `.hick-cache` recording. A
document that passes on macOS therefore passes on Windows, and a recording made
on one serves a run on the other.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/an-expectation-means-the-same-on-every-platform.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
