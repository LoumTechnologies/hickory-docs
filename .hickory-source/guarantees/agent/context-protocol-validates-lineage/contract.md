<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Context protocol validates declared byte lineage before integration

For: Authors using an agent to change documents while retaining review control.

Given a self-contained context v1 witness, when it is checked by Rust
`hick_context_protocol::validate_bundle` or Python
`context_protocol.validate_bundle`, then unknown node references, changed text
hashes, modified request segments, coverage gaps/overlaps and false exact
transformation mappings are rejected with typed errors. Summaries and model/tool
outputs retain typed producer genealogy without claiming exact correspondence or
semantic support. Opaque lineage stays explicit. Immutable artifact descriptors
are checked without pretending the validator fetched their bytes.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/context-protocol-validates-lineage.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
