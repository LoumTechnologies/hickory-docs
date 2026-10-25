<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Ingested Bytes Are Editable And Never Report As Yours: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/api/types.ts](../../../shared/files/apps/web/src/api/types.ts.md)
- [apps/web/src/components/OutputEditorPane.tsx](../../../shared/files/apps/web/src/components/OutputEditorPane.tsx.md)
- [apps/web/src/styles.css](../../../shared/files/apps/web/src/styles.css.md)
- [crates/hick-flow/src/node.rs](../../../shared/files/crates/hick-flow/src/node.rs.md)
- [crates/hick-flow/src/transform.rs](../../../shared/files/crates/hick-flow/src/transform.rs.md)
- [crates/hick-literate/src/lib.rs](../../../shared/files/crates/hick-literate/src/lib.rs.md)
- [crates/hick-literate/src/weave.rs](../../../shared/files/crates/hick-literate/src/weave.rs.md)
- [crates/hickory-agent/src/tools/mod.rs](../../../shared/files/crates/hickory-agent/src/tools/mod.rs.md)
- [crates/hickory-cli/src/main.rs](../../../shared/files/crates/hickory-cli/src/main.rs.md)
- [crates/hickory-lineage/src/lib.rs](../../../shared/files/crates/hickory-lineage/src/lib.rs.md)

## Tests and fixtures

- [crates/hickory-cli/tests/ingest_scaffold.rs](../../../shared/files/crates/hickory-cli/tests/ingest_scaffold.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
