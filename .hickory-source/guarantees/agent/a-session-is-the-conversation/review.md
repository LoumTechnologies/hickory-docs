<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Read the retained branch of an agent conversation: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/ChatDock.tsx](../../../shared/files/apps/web/src/components/ChatDock.tsx.md)
- [apps/web/src/components/ChatTree.tsx](files/apps/web/src/components/ChatTree.tsx.md)
- [apps/web/src/components/SessionTurns.tsx](../../../shared/files/apps/web/src/components/SessionTurns.tsx.md)
- [apps/web/src/editor/folding.ts](../../../shared/files/apps/web/src/editor/folding.ts.md)
- [apps/web/src/editor/wysiwyg.ts](../../../shared/files/apps/web/src/editor/wysiwyg.ts.md)
- [apps/web/src/elements/session/view.tsx](../../../shared/files/apps/web/src/elements/session/view.tsx.md)
- [apps/web/src/views/workspaceTabs.tsx](../../../shared/files/apps/web/src/views/workspaceTabs.tsx.md)
- [crates/hick-lsp/src/session_lint.rs](files/crates/hick-lsp/src/session_lint.rs.md)
- [crates/hickory-agent/src/session.rs](../../../shared/files/crates/hickory-agent/src/session.rs.md)
- [crates/hickory-agent/src/session_view.rs](../../../shared/files/crates/hickory-agent/src/session_view.rs.md)
- [crates/hickory-cli/src/serve/agent.rs](../../../shared/files/crates/hickory-cli/src/serve/agent.rs.md)
- [crates/hickory-cli/src/serve/api.rs](../../../shared/files/crates/hickory-cli/src/serve/api.rs.md)

## Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../shared/files/apps/web/e2e/literate-editor.spec.ts.md)
- [apps/web/src/components/ChatDock.test.tsx](../../../shared/files/apps/web/src/components/ChatDock.test.tsx.md)
- [apps/web/src/editor/folding.test.ts](../../../shared/files/apps/web/src/editor/folding.test.ts.md)
- [apps/web/src/editor/session.test.tsx](files/apps/web/src/editor/session.test.tsx.md)
- [apps/web/src/views/ConversationEditor.test.tsx](../../../support/files/apps/web/src/views/ConversationEditor.test.tsx.md)
- [crates/hickory-cli/tests/lsp_desktop.rs](../../../shared/files/crates/hickory-cli/tests/lsp_desktop.rs.md)
- [crates/hickory-cli/tests/serve_python_agent.rs](../../../shared/files/crates/hickory-cli/tests/serve_python_agent.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
