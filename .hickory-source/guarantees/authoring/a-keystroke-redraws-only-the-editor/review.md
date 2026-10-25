<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Keystroke Redraws Only The Editor: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/debug/useDebugger.ts](../../../shared/files/apps/web/src/debug/useDebugger.ts.md)
- [apps/web/src/editor/DocumentEditor.tsx](../../../shared/files/apps/web/src/editor/DocumentEditor.tsx.md)
- [apps/web/src/editor/EditorRuler.tsx](../../../shared/files/apps/web/src/editor/EditorRuler.tsx.md)
- [apps/web/src/editor/embedded.ts](../../../shared/files/apps/web/src/editor/embedded.ts.md)
- [apps/web/src/lib/offsets.ts](files/apps/web/src/lib/offsets.ts.md)
- [apps/web/src/shell/Ribbons.tsx](../../../shared/files/apps/web/src/shell/Ribbons.tsx.md)
- [apps/web/src/shell/StatusBar.tsx](files/apps/web/src/shell/StatusBar.tsx.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [apps/web/src/views/documentSession.tsx](../../../shared/files/apps/web/src/views/documentSession.tsx.md)

## Tests and fixtures

- [apps/web/src/editor/embedded.test.ts](../../../shared/files/apps/web/src/editor/embedded.test.ts.md)
- [apps/web/src/lib/offsets.test.ts](../../../shared/files/apps/web/src/lib/offsets.test.ts.md)
- [apps/web/src/shell/StatusBar.test.tsx](files/apps/web/src/shell/StatusBar.test.tsx.md)
- [apps/web/src/views/sessionRegistry.test.ts](files/apps/web/src/views/sessionRegistry.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
