<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The engineer tour distinguishes replay from execution: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/WorkflowTour.tsx](../../verification/browser-predictions-use-the-shared-gate/files/apps/web/src/components/WorkflowTour.tsx.md)
- [apps/web/src/embed/DebuggableDocument.tsx](../../../support/files/apps/web/src/embed/DebuggableDocument.tsx.md)
- [apps/web/src/tour/scenario.ts](files/apps/web/src/tour/scenario.ts.md)
- [apps/web/src/views/LandingView.tsx](../../../shared/files/apps/web/src/views/LandingView.tsx.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [crates/hickory-cli/src/serve/engineer_tour.rs](../../../shared/files/crates/hickory-cli/src/serve/engineer_tour.rs.md)

## Tests and fixtures

- [apps/web/e2e/browserEmbedding.spec.ts](../../../shared/files/apps/web/e2e/browserEmbedding.spec.ts.md)
- [apps/web/e2e/privateCoachLayout.spec.ts](../../../shared/files/apps/web/e2e/privateCoachLayout.spec.ts.md)
- [apps/web/src/components/NativeWorkflowTour.test.tsx](../../../shared/files/apps/web/src/components/NativeWorkflowTour.test.tsx.md)
- [apps/web/src/components/WorkflowTour.test.tsx](files/apps/web/src/components/WorkflowTour.test.tsx.md)
- [apps/web/src/tour/scenario.test.ts](files/apps/web/src/tour/scenario.test.ts.md)
- [crates/hickory-cli/tests/engineer_tour.rs](../../../shared/files/crates/hickory-cli/tests/engineer_tour.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
