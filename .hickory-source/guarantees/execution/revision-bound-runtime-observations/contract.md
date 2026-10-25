<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Runtime observations retain their producing revision and occurrence

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a host using `hick-observe`, when a provider supplies runtime evidence and
the host saves it, then retrieval preserves its run/event identity, captured source
bytes and digest, optional block/invocation/thread/task identities, emission and
caller locations, precision, and typed payload. Receive order makes no claim
about causality between threads. Missing emission metadata means unknown.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/revision-bound-runtime-observations.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
