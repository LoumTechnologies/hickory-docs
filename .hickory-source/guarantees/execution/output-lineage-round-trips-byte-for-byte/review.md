<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Carry a generated-text edit back to its local source: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

The local route weaves the current documents, obtains provenance, and calls map_edits. It constructs and parses all updated documents before writing through the local document store. Postgres and a mandatory Git commit are absent from this local path.

## Implementation

- [crates/hickory-cli/src/serve/api.rs](../../../shared/files/crates/hickory-cli/src/serve/api.rs.md)
- [crates/hickory-cli/src/serve/outputs.rs](../../../shared/files/crates/hickory-cli/src/serve/outputs.rs.md)
- [crates/hickory-lineage/src/lib.rs](../../../shared/files/crates/hickory-lineage/src/lib.rs.md)

## Tests and fixtures

- [crates/hick-literate/tests/pipeline_tests.rs](../../../shared/files/crates/hick-literate/tests/pipeline_tests.rs.md)
- [crates/hickory-cli/tests/serve_local.rs](../../../shared/files/crates/hickory-cli/tests/serve_local.rs.md)

## Focused witnesses

- `maps_edit_within_one_span` in `crates/hickory-lineage/src/lib.rs`.
- `rejects_synthetic_overlap_with_offending_range` in `crates/hickory-lineage/src/lib.rs`.

## Evidence status

- Implementation review: partially verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
