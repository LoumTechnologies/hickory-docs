<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Meeting recordings remain recoverable local evidence

For: Authors editing executable documents and the files those documents produce.

Given an explicitly started browser microphone recording,
when capture yields chunks, then they are retained with offset checks, size
limits and content hashes in application data outside the workspace. Stopping
finalizes the recording; interrupted chunks can be recovered, exported and
explicitly deleted. Audio is never automatically committed to Git.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/recordings-stay-local.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
