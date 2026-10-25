<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Conflict actions name the write

For: Authors editing executable documents and the files those documents produce.

Given a diverged file, when its choices are shown, then each button names
its consequence and the visible explanation says which bytes will be written
or discarded. Leaving disk unchanged dismisses a warning without claiming that
divergence is resolved. Regeneration is offered only when an output exists;
otherwise the surface explains how to obtain one.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/conflict-actions-name-the-write.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
