<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Review threads preserve the questioned target: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/ReviewPanel.tsx](../../../support/files/apps/web/src/components/ReviewPanel.tsx.md)
- [apps/web/src/editor/useBlame.ts](../../../shared/files/apps/web/src/editor/useBlame.ts.md)
- [apps/web/src/lib/reviews.ts](../../../support/files/apps/web/src/lib/reviews.ts.md)
- [apps/web/src/shell/Ribbons.tsx](../../../shared/files/apps/web/src/shell/Ribbons.tsx.md)
- [crates/hickory-cli/src/serve/reviews.rs](files/crates/hickory-cli/src/serve/reviews.rs.md)

## Tests and fixtures

- [apps/web/e2e/browserEmbedding.spec.ts](../../../shared/files/apps/web/e2e/browserEmbedding.spec.ts.md)
- [apps/web/src/components/ReviewPanel.test.tsx](../../../shared/files/apps/web/src/components/ReviewPanel.test.tsx.md)
- [apps/web/src/lib/reviews.test.ts](files/apps/web/src/lib/reviews.test.ts.md)
- [crates/hickory-cli/tests/serve_blame.rs](../../../shared/files/crates/hickory-cli/tests/serve_blame.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
