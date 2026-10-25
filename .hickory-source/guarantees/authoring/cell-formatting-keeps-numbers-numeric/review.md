<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Cell formatting keeps numbers numeric: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/MarkdownTable.tsx](../../../support/files/apps/web/src/components/MarkdownTable.tsx.md)
- [apps/web/src/components/TableFormatControl.tsx](../../../support/files/apps/web/src/components/TableFormatControl.tsx.md)
- [apps/web/src/components/TablePanel.tsx](../../../shared/files/apps/web/src/components/TablePanel.tsx.md)
- [apps/web/src/editor/markdownTables.ts](../../../support/files/apps/web/src/editor/markdownTables.ts.md)
- [apps/web/src/editor/rendered.ts](../../../shared/files/apps/web/src/editor/rendered.ts.md)
- [apps/web/src/editor/tableEdit.ts](files/apps/web/src/editor/tableEdit.ts.md)
- [apps/web/src/elements/table/view.tsx](../../../support/files/apps/web/src/elements/table/view.tsx.md)
- [apps/web/src/lib/csv.ts](../a-table-is-a-dataset-and-a-paragraph/files/apps/web/src/lib/csv.ts.md)
- [apps/web/src/lib/tableFormats.ts](files/apps/web/src/lib/tableFormats.ts.md)
- [crates/hick-blocks/src/lib.rs](../../../shared/files/crates/hick-blocks/src/lib.rs.md)
- [crates/hick-literate/src/csv_table.rs](../../../shared/files/crates/hick-literate/src/csv_table.rs.md)

## Tests and fixtures

- [apps/web/e2e/tableFormatting.spec.ts](files/apps/web/e2e/tableFormatting.spec.ts.md)
- [apps/web/src/components/FenceTable.test.tsx](../../../shared/files/apps/web/src/components/FenceTable.test.tsx.md)
- [apps/web/src/components/TableFormatting.test.tsx](files/apps/web/src/components/TableFormatting.test.tsx.md)
- [apps/web/src/components/TablePanel.test.tsx](../../../shared/files/apps/web/src/components/TablePanel.test.tsx.md)
- [apps/web/src/editor/tableFormatting.test.tsx](files/apps/web/src/editor/tableFormatting.test.tsx.md)
- [apps/web/src/lib/tableClipboard.test.ts](../../../shared/files/apps/web/src/lib/tableClipboard.test.ts.md)
- [apps/web/src/lib/tableFormats.test.ts](files/apps/web/src/lib/tableFormats.test.ts.md)
- [crates/hick-formula/tests/backends.rs](../../../shared/files/crates/hick-formula/tests/backends.rs.md)
- [crates/hickory-cli/tests/tables.rs](../../../shared/files/crates/hickory-cli/tests/tables.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
