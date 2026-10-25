<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Session Appears Where It Is Working: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/shell/FolderTreePane.tsx](../../../shared/files/apps/web/src/shell/FolderTreePane.tsx.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [crates/hick-term/src/screen.rs](files/crates/hick-term/src/screen.rs.md)
- [crates/hick-term/src/session.rs](../../../shared/files/crates/hick-term/src/session.rs.md)
- [crates/hick-term/src/shell_integration.rs](../../../shared/files/crates/hick-term/src/shell_integration.rs.md)

## Tests and fixtures

- [apps/web/src/lib/treeTerminals.test.ts](files/apps/web/src/lib/treeTerminals.test.ts.md)
- [apps/web/src/shell/FolderTreePane.test.tsx](../../../shared/files/apps/web/src/shell/FolderTreePane.test.tsx.md)
- [crates/hick-term/tests/a_real_shell_reports_its_directory.rs](files/crates/hick-term/tests/a_real_shell_reports_its_directory.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
