<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Three Provenances Are Drawn Apart, And Any Subset Can Be Shown: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/lib/provenanceLayers.ts](files/apps/web/src/lib/provenanceLayers.ts.md)
- [apps/web/src/shell/ProvenanceToggles.tsx](files/apps/web/src/shell/ProvenanceToggles.tsx.md)
- [apps/web/src/shell/Ribbons.tsx](../../../shared/files/apps/web/src/shell/Ribbons.tsx.md)
- [apps/web/src/styles.css](../../../shared/files/apps/web/src/styles.css.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [crates/hick-handlers/src/handlers/transform.rs](../../../shared/files/crates/hick-handlers/src/handlers/transform.rs.md)
- [crates/hick-literate/src/weave.rs](../../../shared/files/crates/hick-literate/src/weave.rs.md)
- [crates/hickory-cli/src/lib.rs](../../../shared/files/crates/hickory-cli/src/lib.rs.md)
- [crates/hickory-cli/src/main.rs](../../../shared/files/crates/hickory-cli/src/main.rs.md)

## Tests and fixtures

- [apps/web/src/lib/provenanceLayers.test.ts](files/apps/web/src/lib/provenanceLayers.test.ts.md)
- [crates/hickory-cli/tests/cites.rs](../../../shared/files/crates/hickory-cli/tests/cites.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
