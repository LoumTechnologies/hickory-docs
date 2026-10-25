<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Exception Breakpoint Uses The Adapter's Own Filters: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/debug/DebugStrip.tsx](../../../shared/files/apps/web/src/debug/DebugStrip.tsx.md)
- [apps/web/src/debug/client.ts](../../../shared/files/apps/web/src/debug/client.ts.md)
- [apps/web/src/debug/useDebugger.ts](../../../shared/files/apps/web/src/debug/useDebugger.ts.md)
- [crates/hickory-cli/src/serve/debug_bridge.rs](../../../shared/files/crates/hickory-cli/src/serve/debug_bridge.rs.md)

## Tests and fixtures

- [apps/web/src/debug/DebugStrip.test.tsx](../../../shared/files/apps/web/src/debug/DebugStrip.test.tsx.md)
- [apps/web/src/debug/useDebugger.test.ts](../../../shared/files/apps/web/src/debug/useDebugger.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
