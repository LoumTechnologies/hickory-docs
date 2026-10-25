<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Pair identity without silently granting execution: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

Fleet state stores identity, enrolled public keys and verb grants. The peer channel uses that identity when admitting a connection and checks the grants for each request.

## Implementation

- [apps/web/src/views/FleetPane.tsx](../../../shared/files/apps/web/src/views/FleetPane.tsx.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [apps/web/src/views/workspaceState.ts](../../../shared/files/apps/web/src/views/workspaceState.ts.md)
- [crates/hickory-cli/src/main.rs](../../../shared/files/crates/hickory-cli/src/main.rs.md)
- [crates/hickory-cli/src/serve/history.rs](../../../shared/files/crates/hickory-cli/src/serve/history.rs.md)
- [crates/hickory-fleet/src/lib.rs](../../../shared/files/crates/hickory-fleet/src/lib.rs.md)
- [crates/hickory-peer/src/grants.rs](../../../shared/files/crates/hickory-peer/src/grants.rs.md)

## Tests and fixtures

- [apps/web/src/views/FleetPane.test.tsx](../../../shared/files/apps/web/src/views/FleetPane.test.tsx.md)
- [apps/web/src/views/toolTabs.test.ts](files/apps/web/src/views/toolTabs.test.ts.md)
- [crates/hickory-cli/tests/fleet.rs](files/crates/hickory-cli/tests/fleet.rs.md)
- [crates/hickory-fleet/tests/roster_guarded.rs](files/crates/hickory-fleet/tests/roster_guarded.rs.md)

## Evidence status

- Implementation review: partially verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
