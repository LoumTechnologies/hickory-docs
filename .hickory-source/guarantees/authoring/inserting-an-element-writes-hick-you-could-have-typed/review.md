<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Inserting An Element Writes Hick You Could Have Typed Yourself: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/desktop/src-tauri/src/lib.rs](../../../shared/files/apps/desktop/src-tauri/src/lib.rs.md)
- [apps/web/src/App.tsx](../../../shared/files/apps/web/src/App.tsx.md)
- [apps/web/src/components/InsertMenu.tsx](../../../shared/files/apps/web/src/components/InsertMenu.tsx.md)
- [apps/web/src/editor/DocumentEditor.tsx](../../../shared/files/apps/web/src/editor/DocumentEditor.tsx.md)
- [apps/web/src/editor/activeEditor.ts](files/apps/web/src/editor/activeEditor.ts.md)
- [apps/web/src/editor/insertElement.ts](files/apps/web/src/editor/insertElement.ts.md)
- [apps/web/src/lib/insertCatalog.ts](../../../shared/files/apps/web/src/lib/insertCatalog.ts.md)
- [apps/web/src/lib/menuBridge.ts](files/apps/web/src/lib/menuBridge.ts.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)

## Tests and fixtures

- [apps/desktop/src-tauri/tests/insert_menu_matches_catalogue.rs](../../../shared/files/apps/desktop/src-tauri/tests/insert_menu_matches_catalogue.rs.md)
- [apps/web/src/components/InsertMenu.test.tsx](../../../shared/files/apps/web/src/components/InsertMenu.test.tsx.md)
- [apps/web/src/lib/insertCatalog.test.ts](files/apps/web/src/lib/insertCatalog.test.ts.md)
- [apps/web/src/lib/menuBridge.test.ts](files/apps/web/src/lib/menuBridge.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
