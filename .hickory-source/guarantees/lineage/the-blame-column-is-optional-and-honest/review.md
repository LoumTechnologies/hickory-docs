<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Blame Column Is Optional And Honest: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/desktop/src-tauri/src/lib.rs](../../../shared/files/apps/desktop/src-tauri/src/lib.rs.md)
- [apps/web/src/editor/blameGutter.ts](files/apps/web/src/editor/blameGutter.ts.md)
- [apps/web/src/editor/useBlame.ts](../../../shared/files/apps/web/src/editor/useBlame.ts.md)
- [apps/web/src/lib/blamePref.ts](files/apps/web/src/lib/blamePref.ts.md)
- [apps/web/src/lib/menuBridge.ts](../../authoring/inserting-an-element-writes-hick-you-could-have-typed/files/apps/web/src/lib/menuBridge.ts.md)
- [apps/web/src/views/useZoom.ts](../../../support/files/apps/web/src/views/useZoom.ts.md)
- [crates/hickory-cli/src/agent_lineage.rs](../../../shared/files/crates/hickory-cli/src/agent_lineage.rs.md)
- [crates/hickory-cli/src/serve/api.rs](../../../shared/files/crates/hickory-cli/src/serve/api.rs.md)

## Tests and fixtures

- [apps/web/src/editor/blameGutter.controls.test.tsx](../../../shared/files/apps/web/src/editor/blameGutter.controls.test.tsx.md)
- [apps/web/src/editor/blameGutter.test.ts](files/apps/web/src/editor/blameGutter.test.ts.md)
- [crates/hickory-cli/tests/serve_blame.rs](../../../shared/files/crates/hickory-cli/tests/serve_blame.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
