<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Saved means the displayed bytes reached the file: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/lib/outputSave.ts](../../../shared/files/apps/web/src/lib/outputSave.ts.md)
- [apps/web/src/lib/plainFileSave.ts](../../../shared/files/apps/web/src/lib/plainFileSave.ts.md)
- [apps/web/src/lib/viewBuffers.ts](../../../shared/files/apps/web/src/lib/viewBuffers.ts.md)
- [apps/web/src/shell/views.tsx](../../../shared/files/apps/web/src/shell/views.tsx.md)
- [apps/web/src/views/RepresentationPane.tsx](../../../shared/files/apps/web/src/views/RepresentationPane.tsx.md)
- [apps/web/src/views/documentSession.tsx](../../../shared/files/apps/web/src/views/documentSession.tsx.md)
- [apps/web/src/views/useDocumentPersistence.ts](files/apps/web/src/views/useDocumentPersistence.ts.md)
- [apps/web/src/views/useUnsavedLifecycle.ts](../../../shared/files/apps/web/src/views/useUnsavedLifecycle.ts.md)

## Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../shared/files/apps/web/e2e/literate-editor.spec.ts.md)
- [apps/web/src/lib/outputSave.test.ts](../../../shared/files/apps/web/src/lib/outputSave.test.ts.md)
- [apps/web/src/lib/plainFileSave.test.ts](../../../shared/files/apps/web/src/lib/plainFileSave.test.ts.md)
- [apps/web/src/shell/views.test.tsx](../../../shared/files/apps/web/src/shell/views.test.tsx.md)
- [apps/web/src/views/RepresentationPane.test.tsx](../../../shared/files/apps/web/src/views/RepresentationPane.test.tsx.md)
- [apps/web/src/views/useDocumentPersistence.test.tsx](files/apps/web/src/views/useDocumentPersistence.test.tsx.md)
- [apps/web/src/views/useUnsavedLifecycle.test.tsx](../../../shared/files/apps/web/src/views/useUnsavedLifecycle.test.tsx.md)
- [apps/web/src/views/workspaceTabs.test.tsx](../../../shared/files/apps/web/src/views/workspaceTabs.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
