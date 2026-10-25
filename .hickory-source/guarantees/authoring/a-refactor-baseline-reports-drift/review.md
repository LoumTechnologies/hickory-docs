<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Refactor Baseline Reports The Moment An Output Would Move: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/RefactorBadge.tsx](../../../shared/files/apps/web/src/components/RefactorBadge.tsx.md)
- [apps/web/src/lib/refactorSummary.ts](files/apps/web/src/lib/refactorSummary.ts.md)
- [apps/web/src/views/workspaceTabs.tsx](../../../shared/files/apps/web/src/views/workspaceTabs.tsx.md)
- [crates/hickory-cli/src/main.rs](../../../shared/files/crates/hickory-cli/src/main.rs.md)
- [crates/hickory-cli/src/serve/refactor.rs](../../../shared/files/crates/hickory-cli/src/serve/refactor.rs.md)

## Tests and fixtures

- [apps/web/src/lib/refactorSummary.test.ts](files/apps/web/src/lib/refactorSummary.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
