<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Breakpoint That Is Not Bound Yet Is Not Refused: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/debug/cmDebug.ts](../../../shared/files/apps/web/src/debug/cmDebug.ts.md)
- [apps/web/src/styles.css](../../../shared/files/apps/web/src/styles.css.md)
- [crates/hick-dap/src/session.rs](../../../shared/files/crates/hick-dap/src/session.rs.md)
- [crates/hickory-cli/src/serve/debug_bridge.rs](../../../shared/files/crates/hickory-cli/src/serve/debug_bridge.rs.md)

## Tests and fixtures

- [apps/web/src/debug/cmDebug.test.ts](../../../shared/files/apps/web/src/debug/cmDebug.test.ts.md)
- [crates/hick-dap/tests/live_session.rs](../../../shared/files/crates/hick-dap/tests/live_session.rs.md)
- [crates/hick-dap/tests/live_session_csharp.rs](../../../shared/files/crates/hick-dap/tests/live_session_csharp.rs.md)
- [crates/hickory-cli/tests/debug_desktop.rs](../../../shared/files/crates/hickory-cli/tests/debug_desktop.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
