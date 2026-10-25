<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Editor Does Not Wait For Workspace Preparation: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/desktop/src-tauri/src/launch_path.rs](../../../shared/files/apps/desktop/src-tauri/src/launch_path.rs.md)
- [apps/web/src/App.tsx](../../../shared/files/apps/web/src/App.tsx.md)
- [apps/web/src/api/client.ts](../../../shared/files/apps/web/src/api/client.ts.md)
- [apps/web/src/api/workspaceStartup.ts](../../../support/files/apps/web/src/api/workspaceStartup.ts.md)
- [apps/web/src/components/StartupNotice.tsx](../../../support/files/apps/web/src/components/StartupNotice.tsx.md)
- [apps/web/src/lib/focusWhenShown.ts](../../../support/files/apps/web/src/lib/focusWhenShown.ts.md)
- [apps/web/src/main.tsx](../../verification/predictions-precede-managed-observation/files/apps/web/src/main.tsx.md)
- [apps/web/src/views/deferredAgent.tsx](../../../support/files/apps/web/src/views/deferredAgent.tsx.md)
- [crates/hickory-cli/src/engine/daemon.rs](../../../shared/files/crates/hickory-cli/src/engine/daemon.rs.md)
- [crates/hickory-cli/src/engine/startup.rs](files/crates/hickory-cli/src/engine/startup.rs.md)
- [crates/hickory-cli/src/engine/watch.rs](../../../support/files/crates/hickory-cli/src/engine/watch.rs.md)

## Tests and fixtures

- [apps/web/e2e/startup.spec.ts](../../../shared/files/apps/web/e2e/startup.spec.ts.md)
- [apps/web/src/api/workspaceStartup.test.tsx](files/apps/web/src/api/workspaceStartup.test.tsx.md)
- [crates/hickory-cli/tests/engine_lifecycle.rs](../../../shared/files/crates/hickory-cli/tests/engine_lifecycle.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</book:doc>
