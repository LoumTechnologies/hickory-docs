<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Only `hickory-executor-canopy` Knows Cloud Canopy's API

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given that Cloud Canopy's API is under concurrent development, when any code
in this repository talks to canopy (gRPC, sandbox channel protocol, image
path mapping), then that code lives in `crates/hickory-executor-canopy`
and nowhere else; all other code depends only on the `Executor` trait in
`crates/hickory-executor`.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/canopy-api-isolated-to-one-crate.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
