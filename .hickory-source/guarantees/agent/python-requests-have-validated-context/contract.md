<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Python model requests carry validated context manifests

For: Authors using an agent to change documents while retaining review control.

Given the built-in confined Python agent,
when Python selects messages at a model boundary,
then it registers Python constructions, preserves captured read/editor references,
and submits an ordered manifest covering each message's exact UTF-8 bytes, role,
branch, refresh policy and observed cursor. Rust resolves references against its
immutable store and validates complete segment coverage before constructing the
messages passed to the provider adapter. Rust adds no prompt or refresh policy.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/python-requests-have-validated-context.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
