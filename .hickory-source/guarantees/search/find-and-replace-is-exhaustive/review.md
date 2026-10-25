<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Find And Replace Is Exhaustive, And Refuses What It Would Undo: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/api/client.ts](../../../shared/files/apps/web/src/api/client.ts.md)
- [apps/web/src/lib/revealLine.ts](../../../shared/files/apps/web/src/lib/revealLine.ts.md)
- [apps/web/src/shell/TreeFindReplace.tsx](files/apps/web/src/shell/TreeFindReplace.tsx.md)
- [crates/hickory-cli/src/serve/api.rs](../../../shared/files/crates/hickory-cli/src/serve/api.rs.md)
- [crates/hickory-cli/src/serve/find.rs](files/crates/hickory-cli/src/serve/find.rs.md)

## Tests and fixtures

- [apps/web/src/lib/revealLine.test.ts](files/apps/web/src/lib/revealLine.test.ts.md)
- [apps/web/src/shell/TreeFindReplace.test.tsx](files/apps/web/src/shell/TreeFindReplace.test.tsx.md)
- [crates/hickory-cli/tests/serve_find.rs](../../../shared/files/crates/hickory-cli/tests/serve_find.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
