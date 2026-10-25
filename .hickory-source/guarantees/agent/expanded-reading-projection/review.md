<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Selected reading scope remains separate from complete edit state: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/debug/cmRuntime.ts](../../../support/files/apps/web/src/debug/cmRuntime.ts.md)
- [apps/web/src/lib/contextProjection.ts](../../../support/files/apps/web/src/lib/contextProjection.ts.md)
- [crates/hick-literate/src/session_receipts.rs](../../../shared/files/crates/hick-literate/src/session_receipts.rs.md)
- [crates/hickory-cli/src/serve/agent_context.rs](../../../shared/files/crates/hickory-cli/src/serve/agent_context.rs.md)

## Tests and fixtures

- [apps/agent/tests/test_reading_projection.py](files/apps/agent/tests/test_reading_projection.py.md)
- [apps/web/src/debug/ObservationReading.test.tsx](../../../shared/files/apps/web/src/debug/ObservationReading.test.tsx.md)
- [apps/web/src/debug/cmRuntime.controls.test.tsx](files/apps/web/src/debug/cmRuntime.controls.test.tsx.md)
- [apps/web/src/lib/contextProjection.test.ts](files/apps/web/src/lib/contextProjection.test.ts.md)
- [apps/web/src/views/WorkspaceChat.test.tsx](../../../shared/files/apps/web/src/views/WorkspaceChat.test.tsx.md)

## Focused witnesses

- `preserves Unicode source coordinates, duplicate passages and omitted ranges while pins bypass folds` in `apps/web/src/lib/contextProjection.test.ts`.
- `retains fold intent across editor unmounts and refuses stale source coordinates` in `apps/web/src/lib/contextProjection.test.ts`.

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</book:doc>
