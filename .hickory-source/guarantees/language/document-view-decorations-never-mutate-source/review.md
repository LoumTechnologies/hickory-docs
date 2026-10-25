<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Document-View Styling Never Mutates the Source: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/editor/embedded.ts](../../../shared/files/apps/web/src/editor/embedded.ts.md)
- [apps/web/src/editor/hickDoc.ts](../../../shared/files/apps/web/src/editor/hickDoc.ts.md)
- [apps/web/src/editor/wysiwyg.ts](../../../shared/files/apps/web/src/editor/wysiwyg.ts.md)

## Tests and fixtures

- [apps/web/src/editor/DocumentEditor.test.tsx](../../../shared/files/apps/web/src/editor/DocumentEditor.test.tsx.md)
- [apps/web/src/editor/embedded.test.ts](../../../shared/files/apps/web/src/editor/embedded.test.ts.md)
- [apps/web/src/editor/hickDoc.test.ts](../../../shared/files/apps/web/src/editor/hickDoc.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
