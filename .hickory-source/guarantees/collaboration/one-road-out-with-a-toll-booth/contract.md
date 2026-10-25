<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# One Road Out, With A Toll Booth On It — Never An Airgap

For: Engineers using local sessions or pairing their own machines.

Given a machine bought to run an agent on, when it is sealed and pointed at a
broker, then it holds no credential worth stealing and reaches the network only
through one process the engineer runs — and every host that process forwards to
is one the engineer allowed by name.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/one-road-out-with-a-toll-booth.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
