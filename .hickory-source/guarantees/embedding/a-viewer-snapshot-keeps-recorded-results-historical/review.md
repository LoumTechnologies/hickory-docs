<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A viewer snapshot keeps recorded results historical: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/SnapshotExport.tsx](../../../support/files/apps/web/src/components/SnapshotExport.tsx.md)
- [apps/web/src/embed/PublishedSnapshot.tsx](files/apps/web/src/embed/PublishedSnapshot.tsx.md)
- [apps/web/src/embed/snapshot.tsx](../../../support/files/apps/web/src/embed/snapshot.tsx.md)
- [apps/web/vite.site.config.ts](../../../support/files/apps/web/vite.site.config.ts.md)
- [crates/hickory-cli/src/serve/api.rs](../../../shared/files/crates/hickory-cli/src/serve/api.rs.md)

## Tests and fixtures

- [apps/web/e2e/browserEmbedding.spec.ts](../../../shared/files/apps/web/e2e/browserEmbedding.spec.ts.md)
- [apps/web/src/embed/PublishedSnapshot.controls.test.tsx](files/apps/web/src/embed/PublishedSnapshot.controls.test.tsx.md)
- [apps/web/src/embed/PublishedSnapshot.test.ts](files/apps/web/src/embed/PublishedSnapshot.test.ts.md)
- [apps/web/src/embed/SnapshotPage.test.tsx](files/apps/web/src/embed/SnapshotPage.test.tsx.md)
- [crates/hickory-cli/tests/serve_blame.rs](../../../shared/files/crates/hickory-cli/tests/serve_blame.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
