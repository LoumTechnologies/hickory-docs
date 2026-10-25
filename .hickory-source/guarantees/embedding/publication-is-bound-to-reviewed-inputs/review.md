<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Published executable results identify reviewed inputs: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/embed/PublishedSnapshot.tsx](../a-viewer-snapshot-keeps-recorded-results-historical/files/apps/web/src/embed/PublishedSnapshot.tsx.md)
- [apps/web/src/embed/snapshot.tsx](../../../support/files/apps/web/src/embed/snapshot.tsx.md)
- [scripts/publish-spec-results.py](files/scripts/publish-spec-results.py.md)
- [scripts/test-publish-spec-results.py](files/scripts/test-publish-spec-results.py.md)

## Tests and fixtures

- [apps/web/e2e/predictionPublication.spec.ts](files/apps/web/e2e/predictionPublication.spec.ts.md)
- [apps/web/src/embed/PredictionHistory.test.tsx](files/apps/web/src/embed/PredictionHistory.test.tsx.md)
- [apps/web/src/landing/WorkflowRecording.test.tsx](files/apps/web/src/landing/WorkflowRecording.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
