<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Browser-local prediction practice uses the shared gate

For: Reviewers deciding what an output, recording or prediction actually establishes.

The optional browser branch of the existing workflow coach uses
`hick-observe::prediction::Ledger` compiled into the shipped parser/ledger WASM.
It does not implement a second comparison or promotion engine. The portable
journal uses the same validation, seal, admission, original requirement,
classification, interruption, actor-scoped history and promotion methods as the
native private ledger. Native private-file ledgers cannot export raw portable
snapshots through this API.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/browser-predictions-use-the-shared-gate.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
