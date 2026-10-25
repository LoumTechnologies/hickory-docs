<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Typed Text Is Prose, Not A Transcript: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/api/client.ts](../../../shared/files/apps/web/src/api/client.ts.md)
- [apps/web/src/components/ScratchpadPane.tsx](files/apps/web/src/components/ScratchpadPane.tsx.md)
- [apps/web/src/views/workspaceState.ts](../../../shared/files/apps/web/src/views/workspaceState.ts.md)
- [crates/hickory-cli/src/ingest.rs](../../../shared/files/crates/hickory-cli/src/ingest.rs.md)
- [crates/hickory-cli/src/serve/plain_file.rs](../../../shared/files/crates/hickory-cli/src/serve/plain_file.rs.md)

## Tests and fixtures

- [apps/web/src/components/ScratchpadPane.test.tsx](files/apps/web/src/components/ScratchpadPane.test.tsx.md)
- [apps/web/src/router.test.ts](../../../shared/files/apps/web/src/router.test.ts.md)
- [apps/web/src/views/workspaceState.test.ts](../../../shared/files/apps/web/src/views/workspaceState.test.ts.md)
- [crates/hickory-cli/tests/ingest.rs](../../../shared/files/crates/hickory-cli/tests/ingest.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
