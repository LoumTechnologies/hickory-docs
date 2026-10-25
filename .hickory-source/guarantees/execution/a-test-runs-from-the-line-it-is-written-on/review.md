<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Test Runs From The Line It Is Written On: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/OutputEditorPane.tsx](../../../shared/files/apps/web/src/components/OutputEditorPane.tsx.md)
- [apps/web/src/components/PlainFilePane.tsx](../../../shared/files/apps/web/src/components/PlainFilePane.tsx.md)
- [apps/web/src/editor/testGutter.ts](files/apps/web/src/editor/testGutter.ts.md)
- [apps/web/src/lib/revealLine.ts](../../../shared/files/apps/web/src/lib/revealLine.ts.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)

## Tests and fixtures

- [apps/web/src/editor/testGutter.controls.test.tsx](files/apps/web/src/editor/testGutter.controls.test.tsx.md)
- [apps/web/src/editor/testGutter.test.ts](files/apps/web/src/editor/testGutter.test.ts.md)
- [crates/hickory-cli/src/serve/test_run.rs](../../../shared/files/crates/hickory-cli/src/serve/test_run.rs.md)
- [crates/hickory-cli/tests/test_run.rs](files/crates/hickory-cli/tests/test_run.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
