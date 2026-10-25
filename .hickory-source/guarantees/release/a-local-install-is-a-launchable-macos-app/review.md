<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Local Install Is a Launchable macOS App: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/desktop/src-tauri/src/lib.rs](../../../shared/files/apps/desktop/src-tauri/src/lib.rs.md)
- [apps/desktop/src-tauri/src/server.rs](../../../shared/files/apps/desktop/src-tauri/src/server.rs.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [scripts/build-python-agent-runtime.sh](../../../shared/files/scripts/build-python-agent-runtime.sh.md)
- [scripts/default-markdown-editor.rs](files/scripts/default-markdown-editor.rs.md)
- [scripts/dist-desktop.sh](../../../shared/files/scripts/dist-desktop.sh.md)

## Tests and fixtures

- [apps/web/src/App.test.tsx](../../../shared/files/apps/web/src/App.test.tsx.md)
- [apps/web/src/views/toolTabs.test.ts](../../collaboration/a-machine-is-a-keypair/files/apps/web/src/views/toolTabs.test.ts.md)
- [apps/web/src/views/workspaceState.test.ts](../../../shared/files/apps/web/src/views/workspaceState.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
