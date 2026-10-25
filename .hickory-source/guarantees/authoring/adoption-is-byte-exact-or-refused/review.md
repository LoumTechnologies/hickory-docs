<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Adopt a plain file without changing its bytes: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/PlainFilePane.tsx](../../../shared/files/apps/web/src/components/PlainFilePane.tsx.md)
- [apps/web/src/views/workspaceState.ts](../../../shared/files/apps/web/src/views/workspaceState.ts.md)
- [crates/hickory-cli/src/adopt.rs](../../../shared/files/crates/hickory-cli/src/adopt.rs.md)
- [crates/hickory-cli/src/lib.rs](../../../shared/files/crates/hickory-cli/src/lib.rs.md)
- [crates/hickory-cli/src/main.rs](../../../shared/files/crates/hickory-cli/src/main.rs.md)
- [crates/hickory-cli/src/serve/refactor.rs](../../../shared/files/crates/hickory-cli/src/serve/refactor.rs.md)

## Tests and fixtures

- [apps/web/src/views/workspaceState.test.ts](../../../shared/files/apps/web/src/views/workspaceState.test.ts.md)
- [crates/hickory-cli/tests/serve_files.rs](../../../shared/files/crates/hickory-cli/tests/serve_files.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
