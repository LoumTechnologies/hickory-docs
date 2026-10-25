<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Unsent responses survive restart: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/desktop/src-tauri/src/folder_session.rs](../../../shared/files/apps/desktop/src-tauri/src/folder_session.rs.md)
- [apps/web/src/components/ChatDock.tsx](../../../shared/files/apps/web/src/components/ChatDock.tsx.md)
- [apps/web/src/lib/responseRecovery.ts](files/apps/web/src/lib/responseRecovery.ts.md)
- [apps/web/src/views/WorkspaceChat.tsx](files/apps/web/src/views/WorkspaceChat.tsx.md)
- [apps/web/src/views/useUnsavedLifecycle.ts](../../../shared/files/apps/web/src/views/useUnsavedLifecycle.ts.md)
- [crates/hickory-cli/src/serve/agent_context.rs](../../../shared/files/crates/hickory-cli/src/serve/agent_context.rs.md)
- [crates/hickory-cli/src/serve/workspace.rs](files/crates/hickory-cli/src/serve/workspace.rs.md)
- [crates/hickory-workspace/src/lib.rs](../../../shared/files/crates/hickory-workspace/src/lib.rs.md)

## Tests and fixtures

- [apps/web/e2e/responseRecovery.spec.ts](files/apps/web/e2e/responseRecovery.spec.ts.md)
- [apps/web/src/components/ChatDock.test.tsx](../../../shared/files/apps/web/src/components/ChatDock.test.tsx.md)
- [apps/web/src/lib/responseRecovery.test.ts](files/apps/web/src/lib/responseRecovery.test.ts.md)
- [apps/web/src/views/WorkspaceChat.test.tsx](../../../shared/files/apps/web/src/views/WorkspaceChat.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
