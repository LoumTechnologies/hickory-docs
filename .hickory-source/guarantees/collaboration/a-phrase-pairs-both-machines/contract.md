<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# One Spoken Phrase Enrols Both Machines, With No Server And Nobody Trusted

For: Engineers using local sessions or pairing their own machines.

Given two machines that have never met, when a phrase generated on one is typed
on the other, then each ends up holding the other's public key — **both
directions in one exchange** — and no server, directory or third party is
involved in deciding whose key is whose.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/a-phrase-pairs-both-machines.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
