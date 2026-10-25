<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Lineage Browser Shows One Column Per Stage: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/lineage/build.ts](files/apps/web/src/lineage/build.ts.md)
- [apps/web/src/lineage/model.ts](files/apps/web/src/lineage/model.ts.md)
- [crates/hick-structure/src/lib.rs](files/crates/hick-structure/src/lib.rs.md)

## Tests and fixtures

- [apps/web/src/landing/demos/demos.test.tsx](../../../shared/files/apps/web/src/landing/demos/demos.test.tsx.md)
- [apps/web/src/lineage/LineageColumns.test.tsx](../../agent/scoped-reading-surfaces-share-native-actions/files/apps/web/src/lineage/LineageColumns.test.tsx.md)
- [apps/web/src/lineage/build.test.ts](../../../support/files/apps/web/src/lineage/build.test.ts.md)
- [apps/web/src/lineage/model.test.ts](../../../support/files/apps/web/src/lineage/model.test.ts.md)
- [apps/web/src/router.test.ts](../../../shared/files/apps/web/src/router.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve.

</book:doc>
