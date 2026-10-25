<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Plain File Has The Same Debugger A Document Has: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/PlainFilePane.tsx](../../../shared/files/apps/web/src/components/PlainFilePane.tsx.md)
- [apps/web/src/debug/cmDebug.ts](../../../shared/files/apps/web/src/debug/cmDebug.ts.md)
- [apps/web/src/debug/plainDebugHosts.tsx](../launch-profiles-choose-targets-within-the-viewed-project/files/apps/web/src/debug/plainDebugHosts.tsx.md)
- [apps/web/src/debug/useDebugger.ts](../../../shared/files/apps/web/src/debug/useDebugger.ts.md)
- [apps/web/src/lib/pausedElsewhere.ts](../../../support/files/apps/web/src/lib/pausedElsewhere.ts.md)
- [crates/hick-dap/src/build.rs](../../../shared/files/crates/hick-dap/src/build.rs.md)
- [crates/hick-dap/src/session.rs](../../../shared/files/crates/hick-dap/src/session.rs.md)
- [crates/hick-plugins/src/build.rs](files/crates/hick-plugins/src/build.rs.md)
- [crates/hickory-cli/src/debug_sessions.rs](../../../shared/files/crates/hickory-cli/src/debug_sessions.rs.md)
- [crates/hickory-cli/src/serve/debug_bridge.rs](../../../shared/files/crates/hickory-cli/src/serve/debug_bridge.rs.md)
- [crates/hickory-cli/src/serve/socket.rs](../../../shared/files/crates/hickory-cli/src/serve/socket.rs.md)

## Tests and fixtures

- [apps/web/src/components/PlainFilePane.test.tsx](../../../shared/files/apps/web/src/components/PlainFilePane.test.tsx.md)
- [apps/web/src/debug/DebugStrip.test.tsx](../../../shared/files/apps/web/src/debug/DebugStrip.test.tsx.md)
- [apps/web/src/debug/cmDebug.test.ts](../../../shared/files/apps/web/src/debug/cmDebug.test.ts.md)
- [apps/web/src/debug/useDebugger.test.ts](../../../shared/files/apps/web/src/debug/useDebugger.test.ts.md)
- [crates/hick-dap/src/session/tests/tests.rs](files/crates/hick-dap/src/session/tests/tests.rs.md)
- [crates/hickory-cli/tests/debug_desktop.rs](../../../shared/files/crates/hickory-cli/tests/debug_desktop.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
