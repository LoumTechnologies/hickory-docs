<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Comparison edit permissions do not establish replay evidence: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/lib/projectComparison.ts](../../../support/files/apps/web/src/lib/projectComparison.ts.md)
- [crates/hick-lang/src/project/comparison.rs](files/crates/hick-lang/src/project/comparison.rs.md)
- [crates/hickory-cli/src/serve/merged_write.rs](../../../support/files/crates/hickory-cli/src/serve/merged_write.rs.md)

## Tests and fixtures

- [apps/web/src/components/ComparisonEditor.test.tsx](../../../shared/files/apps/web/src/components/ComparisonEditor.test.tsx.md)
- [apps/web/src/lib/projectComparison.test.ts](../../../shared/files/apps/web/src/lib/projectComparison.test.ts.md)
- [crates/hick-lang/tests/project.rs](../../../shared/files/crates/hick-lang/tests/project.rs.md)
- [crates/hickory-cli/tests/merged_view.rs](../../../shared/files/crates/hickory-cli/tests/merged_view.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
