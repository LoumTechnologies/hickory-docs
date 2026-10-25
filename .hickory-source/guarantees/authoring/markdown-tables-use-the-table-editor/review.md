<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Markdown tables use the table editor: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/MarkdownTable.tsx](../../../support/files/apps/web/src/components/MarkdownTable.tsx.md)
- [apps/web/src/editor/cards.ts](../../../shared/files/apps/web/src/editor/cards.ts.md)
- [apps/web/src/editor/markdownTables.ts](../../../support/files/apps/web/src/editor/markdownTables.ts.md)
- [apps/web/src/editor/rendered.ts](../../../shared/files/apps/web/src/editor/rendered.ts.md)
- [apps/web/src/elements/index.ts](../../../shared/files/apps/web/src/elements/index.ts.md)
- [apps/web/src/lib/markdownTable.ts](../../../support/files/apps/web/src/lib/markdownTable.ts.md)

## Tests and fixtures

- [apps/web/src/components/MarkdownTable.test.tsx](files/apps/web/src/components/MarkdownTable.test.tsx.md)
- [apps/web/src/editor/markdownTables.test.tsx](files/apps/web/src/editor/markdownTables.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</book:doc>
