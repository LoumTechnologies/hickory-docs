<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The daemon exits with its last client

For: Authors editing executable documents and the files those documents produce.

Given a local Hickory daemon, when its last desktop window or CLI client
disconnects, then the daemon shuts down. A surviving client keeps the shared
daemon alive. Open terminals, workspace runs, and private investigations do not
independently extend its lifetime.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-daemon-exits-with-its-last-client.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
