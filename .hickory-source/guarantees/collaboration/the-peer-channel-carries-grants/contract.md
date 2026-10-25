<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Peer Channel Admits Only Keys You Hold, And Serves Them Only What Their Grants Allow

For: Engineers using local sessions or pairing their own machines.

Given a machine paired into this fleet, when it connects to this session, then
the connection is authenticated as its ed25519 public key by the transport
itself, checked against the mutual key list, and every request it makes is
gated on that machine's grants before anything reaches this session's server.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/the-peer-channel-carries-grants.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
