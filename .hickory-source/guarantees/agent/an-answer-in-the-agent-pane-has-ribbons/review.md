<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Trace conversation passages without confusing context with support: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/ChatDock.tsx](../../../shared/files/apps/web/src/components/ChatDock.tsx.md)
- [apps/web/src/editor/rendered.ts](../../../shared/files/apps/web/src/editor/rendered.ts.md)
- [apps/web/src/elements/session/view.tsx](../../../shared/files/apps/web/src/elements/session/view.tsx.md)
- [apps/web/src/lib/conversationLineage.ts](files/apps/web/src/lib/conversationLineage.ts.md)
- [apps/web/src/lib/lensSources.ts](files/apps/web/src/lib/lensSources.ts.md)
- [apps/web/src/views/SessionLens.tsx](../../../shared/files/apps/web/src/views/SessionLens.tsx.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [crates/hick-blocks/src/lib.rs](../../../shared/files/crates/hick-blocks/src/lib.rs.md)
- [crates/hick-literate/src/session_elements.rs](files/crates/hick-literate/src/session_elements.rs.md)
- [crates/hick-literate/src/session_receipts.rs](../../../shared/files/crates/hick-literate/src/session_receipts.rs.md)
- [crates/hickory-cli/src/serve/api.rs](../../../shared/files/crates/hickory-cli/src/serve/api.rs.md)
- [crates/hickory-cli/src/serve/sessions.rs](files/crates/hickory-cli/src/serve/sessions.rs.md)

## Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../shared/files/apps/web/e2e/literate-editor.spec.ts.md)
- [apps/web/src/elements/index.test.ts](../../language/an-element-is-drawn-by-its-view/files/apps/web/src/elements/index.test.ts.md)
- [apps/web/src/lib/conversationLineage.test.ts](files/apps/web/src/lib/conversationLineage.test.ts.md)
- [apps/web/src/lib/editorBufferTargets.test.ts](files/apps/web/src/lib/editorBufferTargets.test.ts.md)
- [apps/web/src/lib/lensSources.test.ts](files/apps/web/src/lib/lensSources.test.ts.md)
- [apps/web/src/lib/offsets.test.ts](../../../shared/files/apps/web/src/lib/offsets.test.ts.md)
- [apps/web/src/shell/linkShapes.test.ts](../../../shared/files/apps/web/src/shell/linkShapes.test.ts.md)
- [apps/web/src/views/ConversationEditor.test.tsx](../../../support/files/apps/web/src/views/ConversationEditor.test.tsx.md)
- [apps/web/src/views/SessionLens.test.tsx](../../../shared/files/apps/web/src/views/SessionLens.test.tsx.md)
- [apps/web/src/views/workspaceRibbonSources.test.ts](files/apps/web/src/views/workspaceRibbonSources.test.ts.md)
- [crates/hickory-cli/tests/session_lens.rs](files/crates/hickory-cli/tests/session_lens.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
