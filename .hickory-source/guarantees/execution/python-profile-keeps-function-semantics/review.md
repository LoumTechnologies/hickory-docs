<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Python profiles keep function scope and measurement semantics: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hickory-cli/src/observation.rs](../../../shared/files/crates/hickory-cli/src/observation.rs.md)

## Tests and fixtures

- [apps/web/e2e/browserEmbedding.spec.ts](../../../shared/files/apps/web/e2e/browserEmbedding.spec.ts.md)
- [apps/web/src/debug/decodeRuntimeRun.test.ts](files/apps/web/src/debug/decodeRuntimeRun.test.ts.md)
- [apps/web/src/debug/profileRows.test.ts](files/apps/web/src/debug/profileRows.test.ts.md)
- [apps/web/src/debug/runtimeHistory.test.tsx](../../../shared/files/apps/web/src/debug/runtimeHistory.test.tsx.md)
- [crates/hickory-cli/tests/python_agent_profile.rs](files/crates/hickory-cli/tests/python_agent_profile.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
