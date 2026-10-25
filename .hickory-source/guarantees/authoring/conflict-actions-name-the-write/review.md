<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Conflict actions name the write: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/DivergedBanner.tsx](files/apps/web/src/components/DivergedBanner.tsx.md)
- [apps/web/src/components/PlainFilePane.tsx](../../../shared/files/apps/web/src/components/PlainFilePane.tsx.md)
- [apps/web/src/shell/views.tsx](../../../shared/files/apps/web/src/shell/views.tsx.md)

## Tests and fixtures

- [apps/web/src/components/DivergedBanner.test.tsx](../../../shared/files/apps/web/src/components/DivergedBanner.test.tsx.md)
- [apps/web/src/lib/plainFileSave.test.ts](../../../shared/files/apps/web/src/lib/plainFileSave.test.ts.md)
- [apps/web/src/views/workspaceTabs.test.tsx](../../../shared/files/apps/web/src/views/workspaceTabs.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
