<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Carry a generated-text edit back to its local source

For: Document authors editing a generated text file and maintainers reviewing where that edit lands.

Edit a literal range of generated app.py. Hickory maps the change back to its document; editing transcript output must be refused instead of inventing source.

## What must happen

Given a generated text output with byte-precise literal provenance, the local output-edit route maps supported edits back to their contributing source documents. Synthetic or ambiguous mappings are refused. The route reweaves current source, maps the requested output edits, and parses the proposed documents before applying them. The mapping promise covers byte-precise ranges; it does not make encoded binary entries reverse-editable.


## Limits of the evidence

The mapping unit tests do not certify HTTP concurrency or multi-document crash atomicity. HTTP and UI witnesses remain separate. The more general round-trip contract remains under lineage/ok-mapping-round-trips-byte-exact.md.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/output-lineage-round-trips-byte-for-byte.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Replace the obsolete Postgres/hosted-last-run wording with the inspected local route. Preserve the hosted paragraph in its original evidence file.

Implementation review: partially verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Implementation

## Implementation review

The local route weaves the current documents, obtains provenance, and calls map_edits. It constructs and parses all updated documents before writing through the local document store. Postgres and a mandatory Git commit are absent from this local path.

### Implementation

- [crates/hickory-cli/src/serve/api.rs](../../../mechanisms/crates/hickory-cli/src/serve/api.rs.md#output-e71b04ae5298)
- [crates/hickory-cli/src/serve/outputs.rs](../../../mechanisms/crates/hickory-cli/src/serve/outputs.rs.md#output-c3e5a56f4fd1)
- [crates/hickory-lineage/src/lib.rs](../../../mechanisms/crates/hickory-lineage/src/lib.rs.md#output-bc43b0352ff3)

### Tests and fixtures

- [crates/hick-literate/tests/pipeline_tests.rs](../../../mechanisms/crates/hick-literate/tests/pipeline_tests.rs.md#output-a5d24a6d8973)
- [crates/hickory-cli/tests/serve_local.rs](../../../mechanisms/crates/hickory-cli/tests/serve_local.rs.md#output-2b0b4fa19188)

### Focused witnesses

- `maps_edit_within_one_span` in `crates/hickory-lineage/src/lib.rs`.
- `rejects_synthetic_overlap_with_offending_range` in `crates/hickory-lineage/src/lib.rs`.

### Evidence status

- Implementation review: partially verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
