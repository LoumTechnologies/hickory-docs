<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A fill handle extends formulas: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/TableFormatControl.tsx](../../../support/files/apps/web/src/components/TableFormatControl.tsx.md)
- [apps/web/src/components/TablePanel.tsx](../../../shared/files/apps/web/src/components/TablePanel.tsx.md)
- [apps/web/src/components/useTableFill.ts](files/apps/web/src/components/useTableFill.ts.md)
- [apps/web/src/lib/formulaReferences.ts](files/apps/web/src/lib/formulaReferences.ts.md)
- [apps/web/src/lib/tableFill.ts](files/apps/web/src/lib/tableFill.ts.md)

## Tests and fixtures

- [apps/web/e2e/tableFill.spec.ts](files/apps/web/e2e/tableFill.spec.ts.md)
- [apps/web/src/components/TableFill.test.tsx](files/apps/web/src/components/TableFill.test.tsx.md)
- [apps/web/src/editor/tableFill.test.tsx](files/apps/web/src/editor/tableFill.test.tsx.md)
- [apps/web/src/lib/tableFill.test.ts](files/apps/web/src/lib/tableFill.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
