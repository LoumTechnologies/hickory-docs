<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Markdown Quotes And Task Lists Are Read As Prose: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/OutputEditorPane.tsx](../../../shared/files/apps/web/src/components/OutputEditorPane.tsx.md)
- [apps/web/src/components/PlainFilePane.tsx](../../../shared/files/apps/web/src/components/PlainFilePane.tsx.md)
- [apps/web/src/editor/DocumentEditor.tsx](../../../shared/files/apps/web/src/editor/DocumentEditor.tsx.md)
- [apps/web/src/editor/hickDoc.ts](../../../shared/files/apps/web/src/editor/hickDoc.ts.md)
- [apps/web/src/editor/markdownStyling.ts](files/apps/web/src/editor/markdownStyling.ts.md)
- [apps/web/src/editor/taskList.ts](../../../shared/files/apps/web/src/editor/taskList.ts.md)
- [apps/web/src/styles.css](../../../shared/files/apps/web/src/styles.css.md)

## Tests and fixtures

- [apps/web/src/editor/markdownStyling.test.ts](files/apps/web/src/editor/markdownStyling.test.ts.md)
- [apps/web/src/editor/taskList.test.ts](files/apps/web/src/editor/taskList.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
