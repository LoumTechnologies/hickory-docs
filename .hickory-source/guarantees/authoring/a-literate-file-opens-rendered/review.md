<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Literate File Opens Rendered, And Its Source Is One Click Away: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/CellPanel.tsx](../../../shared/files/apps/web/src/components/CellPanel.tsx.md)
- [apps/web/src/editor/CardRail.tsx](../../../shared/files/apps/web/src/editor/CardRail.tsx.md)
- [apps/web/src/editor/DocumentEditor.tsx](../../../shared/files/apps/web/src/editor/DocumentEditor.tsx.md)
- [apps/web/src/editor/rendered.ts](../../../shared/files/apps/web/src/editor/rendered.ts.md)

## Tests and fixtures

- [apps/web/src/editor/DocumentEditor.test.tsx](../../../shared/files/apps/web/src/editor/DocumentEditor.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
