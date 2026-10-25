<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An iframe validates its host

For: Host developers embedding the document editor without a native execution service.

Given a static document iframe configured with a concrete parent origin,
when a versioned embedding message arrives,
then it is accepted only from its actual parent window at that exact origin.
Load supplies source/revision and optional asset URLs; Save returns the current
source for the host to persist. Disposal unmounts the editor and terminates its
debug worker. No credentials are sent to wildcard origins. Storage remains the
host's responsibility when iframe-local persistence is unavailable.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/embedding/an-iframe-validates-its-host.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
