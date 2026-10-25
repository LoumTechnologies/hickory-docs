<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Mappable Output Edit Round-Trips Byte-For-Byte; Everything Else Is Rejected With A Reason

For: Authors editing generated text and reviewers tracing a result to its source.

Given a generated output file with provenance, when an edit at ANY character
position is submitted, then either `map_edits` returns source edits that —
applied to the source documents and re-woven — reproduce the edited output
byte-for-byte, or it rejects the edit with a provable reason: the range
touches synthetic bytes (separators, derived values, provenance gaps), or its
source bytes are woven into more than one place (a copy pasted twice, where
editing one occurrence cannot be reproduced exactly).

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/ok-mapping-round-trips-byte-exact.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
