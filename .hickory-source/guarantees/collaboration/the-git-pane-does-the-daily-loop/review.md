<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Git Pane Does The Daily Loop: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/api/client.ts](../../../shared/files/apps/web/src/api/client.ts.md)
- [apps/web/src/components/DiffView.tsx](files/apps/web/src/components/DiffView.tsx.md)
- [apps/web/src/views/GitPane.tsx](../../../shared/files/apps/web/src/views/GitPane.tsx.md)
- [crates/hickory-cli/src/serve/git_ops.rs](files/crates/hickory-cli/src/serve/git_ops.rs.md)
- [crates/hickory-cli/src/serve/mod.rs](../../../shared/files/crates/hickory-cli/src/serve/mod.rs.md)

## Tests and fixtures

- [apps/web/src/components/DiffView.test.tsx](files/apps/web/src/components/DiffView.test.tsx.md)
- [apps/web/src/views/GitPane.test.tsx](../../../shared/files/apps/web/src/views/GitPane.test.tsx.md)
- [crates/hickory-cli/tests/git_ops.rs](../../../shared/files/crates/hickory-cli/tests/git_ops.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
