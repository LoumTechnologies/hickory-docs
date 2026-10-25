<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Every Shortcut Is A Setting: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/desktop/src-tauri/src/lib.rs](../../../shared/files/apps/desktop/src-tauri/src/lib.rs.md)
- [apps/web/src/debug/DebugStrip.tsx](../../../shared/files/apps/web/src/debug/DebugStrip.tsx.md)
- [apps/web/src/editor/multiCursor.ts](../../../shared/files/apps/web/src/editor/multiCursor.ts.md)
- [apps/web/src/lib/keymap.ts](../../../shared/files/apps/web/src/lib/keymap.ts.md)
- [apps/web/src/lsp/cmLspFeatures.ts](../save-can-format-first/files/apps/web/src/lsp/cmLspFeatures.ts.md)
- [apps/web/src/main.tsx](../../verification/predictions-precede-managed-observation/files/apps/web/src/main.tsx.md)
- [apps/web/src/shell/ShellView.tsx](../../../support/files/apps/web/src/shell/ShellView.tsx.md)
- [apps/web/src/shell/dired.ts](../../authoring/the-tree-is-a-dired/files/apps/web/src/shell/dired.ts.md)
- [apps/web/src/shell/useTreeNavigation.ts](../../../support/files/apps/web/src/shell/useTreeNavigation.ts.md)
- [apps/web/src/views/KeyboardSection.tsx](files/apps/web/src/views/KeyboardSection.tsx.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [crates/hickory-cli/src/serve/api.rs](../../../shared/files/crates/hickory-cli/src/serve/api.rs.md)
- [crates/hickory-cli/src/serve/mod.rs](../../../shared/files/crates/hickory-cli/src/serve/mod.rs.md)

## Tests and fixtures

- [apps/web/src/debug/DebugStrip.test.tsx](../../../shared/files/apps/web/src/debug/DebugStrip.test.tsx.md)
- [apps/web/src/lib/keymap.test.ts](files/apps/web/src/lib/keymap.test.ts.md)
- [apps/web/src/shell/dired.test.ts](../../authoring/the-tree-is-a-dired/files/apps/web/src/shell/dired.test.ts.md)
- [apps/web/src/views/KeyboardSection.controls.test.tsx](files/apps/web/src/views/KeyboardSection.controls.test.tsx.md)
- [crates/hickory-cli/tests/serve_ui_settings.rs](../../../shared/files/crates/hickory-cli/tests/serve_ui_settings.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
