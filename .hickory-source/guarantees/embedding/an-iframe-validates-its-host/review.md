<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An iframe validates its host: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/embed/EditingSurface.tsx](files/apps/web/src/embed/EditingSurface.tsx.md)
- [apps/web/src/embed/iframe.tsx](files/apps/web/src/embed/iframe.tsx.md)
- [apps/web/src/embed/uiHostSession.ts](../../../shared/files/apps/web/src/embed/uiHostSession.ts.md)

## Tests and fixtures

- [apps/web/e2e/browserEmbedding.spec.ts](../../../shared/files/apps/web/e2e/browserEmbedding.spec.ts.md)
- [apps/web/src/embed/uiHostSession.test.tsx](../../../shared/files/apps/web/src/embed/uiHostSession.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
