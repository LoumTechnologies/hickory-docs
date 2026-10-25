<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Debug the latest atomic project that still contains the displayed bytes: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/ComparisonTools.tsx](files/apps/web/src/components/ComparisonTools.tsx.md)
- [apps/web/src/debug/GeneratedDebugTools.tsx](files/apps/web/src/debug/GeneratedDebugTools.tsx.md)
- [apps/web/src/debug/documentDebugContext.ts](files/apps/web/src/debug/documentDebugContext.ts.md)
- [apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm](../../../shared/files/apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm.md)
- [apps/web/src/elements/patch/Editor.tsx](../../../shared/files/apps/web/src/elements/patch/Editor.tsx.md)
- [apps/web/src/views/documentSession.tsx](../../../shared/files/apps/web/src/views/documentSession.tsx.md)
- [crates/hick-lang/src/project/debug.rs](files/crates/hick-lang/src/project/debug.rs.md)
- [crates/hick-lang/src/project/evaluate.rs](../../../shared/files/crates/hick-lang/src/project/evaluate.rs.md)
- [crates/hick-lsp/src/document.rs](../../../shared/files/crates/hick-lsp/src/document.rs.md)
- [crates/hick-lsp/src/project.rs](../../../shared/files/crates/hick-lsp/src/project.rs.md)
- [crates/hickory-cli/src/debug_sessions.rs](../../../shared/files/crates/hickory-cli/src/debug_sessions.rs.md)
- [crates/hickory-cli/src/project_documents.rs](../../../shared/files/crates/hickory-cli/src/project_documents.rs.md)

## Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../shared/files/apps/web/e2e/literate-editor.spec.ts.md)
- [apps/web/src/components/ComparisonTools.test.tsx](../../../shared/files/apps/web/src/components/ComparisonTools.test.tsx.md)
- [apps/web/src/debug/documentDebugContext.test.ts](files/apps/web/src/debug/documentDebugContext.test.ts.md)
- [apps/web/src/debug/useDebugger.test.ts](../../../shared/files/apps/web/src/debug/useDebugger.test.ts.md)
- [crates/hick-lang/tests/project_debug.rs](../../../shared/files/crates/hick-lang/tests/project_debug.rs.md)
- [crates/hick-lsp/tests/project.rs](../../../shared/files/crates/hick-lsp/tests/project.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
