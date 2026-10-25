<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Carry a generated-text edit back to its local source

For: Document authors editing a generated text file and maintainers reviewing where that edit lands.

Edit a literal range of generated app.py. Hickory maps the change back to its document; editing transcript output must be refused instead of inventing source.

## What must happen

Given a generated text output with byte-precise literal provenance, the local output-edit route maps supported edits back to their contributing source documents. Synthetic or ambiguous mappings are refused. The route reweaves current source, maps the requested output edits, and parses the proposed documents before applying them. The mapping promise covers byte-precise ranges; it does not make encoded binary entries reverse-editable.


## Limits of the evidence

The mapping unit tests do not certify HTTP concurrency or multi-document crash atomicity. HTTP and UI witnesses remain separate. The more general round-trip contract remains under lineage/ok-mapping-round-trips-byte-exact.md.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/output-lineage-round-trips-byte-for-byte.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Replace the obsolete Postgres/hosted-last-run wording with the inspected local route. Preserve the hosted paragraph in its original evidence file.

Implementation review: partially verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
