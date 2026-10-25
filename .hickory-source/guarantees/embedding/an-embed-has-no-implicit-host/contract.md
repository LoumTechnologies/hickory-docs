<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An embed has no implicit host

For: Host developers embedding the document editor without a native execution service.

Given a host-controlled `DocumentEmbed` supplied with source, revision and path,
when it mounts, edits, receives host updates, or unmounts,
then it opens no API connection, socket, executor or analytics client. Its parser
loads as a static WASM asset. Changes carry the host's base revision; a host-driven
replacement is not echoed as a user edit. Two instances keep independent buffers,
registries, selections and disposal. Unsupported elements remain in the source.
Assets are resolved only through an explicit host callback.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/embedding/an-embed-has-no-implicit-host.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
