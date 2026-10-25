<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Breakpoint Can Carry A Condition: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/debug/client.ts](../../../shared/files/apps/web/src/debug/client.ts.md)
- [apps/web/src/debug/cmDebug.ts](../../../shared/files/apps/web/src/debug/cmDebug.ts.md)
- [apps/web/src/debug/useDebugger.ts](../../../shared/files/apps/web/src/debug/useDebugger.ts.md)
- [crates/hick-dap/src/session.rs](../../../shared/files/crates/hick-dap/src/session.rs.md)

## Tests and fixtures

- [apps/web/src/debug/breakpointForm.test.tsx](files/apps/web/src/debug/breakpointForm.test.tsx.md)
- [apps/web/src/debug/cmDebug.test.ts](../../../shared/files/apps/web/src/debug/cmDebug.test.ts.md)
- [apps/web/src/debug/useDebugger.test.ts](../../../shared/files/apps/web/src/debug/useDebugger.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
