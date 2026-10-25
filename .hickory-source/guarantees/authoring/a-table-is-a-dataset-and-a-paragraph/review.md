<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Table Is A Dataset And A Paragraph: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/ContextMenu.tsx](files/apps/web/src/components/ContextMenu.tsx.md)
- [apps/web/src/components/FenceTable.tsx](files/apps/web/src/components/FenceTable.tsx.md)
- [apps/web/src/components/TablePanel.tsx](../../../shared/files/apps/web/src/components/TablePanel.tsx.md)
- [apps/web/src/components/TableSizeDialog.tsx](files/apps/web/src/components/TableSizeDialog.tsx.md)
- [apps/web/src/editor/CardRail.tsx](../../../shared/files/apps/web/src/editor/CardRail.tsx.md)
- [apps/web/src/editor/DocumentEditor.tsx](../../../shared/files/apps/web/src/editor/DocumentEditor.tsx.md)
- [apps/web/src/editor/cards.ts](../../../shared/files/apps/web/src/editor/cards.ts.md)
- [apps/web/src/editor/rendered.ts](../../../shared/files/apps/web/src/editor/rendered.ts.md)
- [apps/web/src/lib/csv.ts](files/apps/web/src/lib/csv.ts.md)
- [apps/web/src/lib/insertCatalog.ts](../../../shared/files/apps/web/src/lib/insertCatalog.ts.md)
- [apps/web/src/lib/railActions.ts](../../../shared/files/apps/web/src/lib/railActions.ts.md)
- [apps/web/src/lib/tableClipboard.ts](files/apps/web/src/lib/tableClipboard.ts.md)
- [apps/web/src/lib/tableMenu.ts](files/apps/web/src/lib/tableMenu.ts.md)
- [apps/web/src/lib/tableSelection.ts](files/apps/web/src/lib/tableSelection.ts.md)
- [apps/web/src/lib/uiState.ts](../../../shared/files/apps/web/src/lib/uiState.ts.md)
- [apps/web/src/shell/TreeContextMenu.tsx](../a-tree-row-opens-in-the-platform/files/apps/web/src/shell/TreeContextMenu.tsx.md)
- [apps/web/src/styles.css](../../../shared/files/apps/web/src/styles.css.md)
- [crates/hick-literate/src/csv_table.rs](../../../shared/files/crates/hick-literate/src/csv_table.rs.md)
- [crates/hick-literate/src/lib.rs](../../../shared/files/crates/hick-literate/src/lib.rs.md)
- [crates/hick-literate/src/render.rs](../../../shared/files/crates/hick-literate/src/render.rs.md)
- [crates/hick-literate/src/weave.rs](../../../shared/files/crates/hick-literate/src/weave.rs.md)

## Tests and fixtures

- [apps/web/src/components/FenceTable.test.tsx](../../../shared/files/apps/web/src/components/FenceTable.test.tsx.md)
- [apps/web/src/components/TablePanel.test.tsx](../../../shared/files/apps/web/src/components/TablePanel.test.tsx.md)
- [apps/web/src/editor/rendered.test.ts](../../../shared/files/apps/web/src/editor/rendered.test.ts.md)
- [apps/web/src/lib/csv.test.ts](files/apps/web/src/lib/csv.test.ts.md)
- [apps/web/src/lib/tableClipboard.test.ts](../../../shared/files/apps/web/src/lib/tableClipboard.test.ts.md)
- [apps/web/src/lib/tableMenu.test.ts](files/apps/web/src/lib/tableMenu.test.ts.md)
- [apps/web/src/lib/tableSelection.test.ts](files/apps/web/src/lib/tableSelection.test.ts.md)
- [apps/web/src/lib/uiState.test.ts](../../../shared/files/apps/web/src/lib/uiState.test.ts.md)
- [apps/web/src/styles.test.ts](../../../shared/files/apps/web/src/styles.test.ts.md)
- [crates/hickory-cli/tests/tables.rs](../../../shared/files/crates/hickory-cli/tests/tables.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
