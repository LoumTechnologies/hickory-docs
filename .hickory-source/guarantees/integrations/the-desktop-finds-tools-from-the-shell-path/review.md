<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Find installed tools when the desktop starts outside a terminal: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/desktop/src-tauri/src/launch_path.rs](../../../shared/files/apps/desktop/src-tauri/src/launch_path.rs.md)
- [apps/desktop/src-tauri/src/main.rs](../../../shared/files/apps/desktop/src-tauri/src/main.rs.md)
- [crates/hick-term/src/session.rs](../../../shared/files/crates/hick-term/src/session.rs.md)
- [crates/hick-term/src/shell_integration.rs](../../../shared/files/crates/hick-term/src/shell_integration.rs.md)
- [crates/hickory-cli/src/engine/client.rs](../../../shared/files/crates/hickory-cli/src/engine/client.rs.md)
- [crates/hickory-cli/src/serve/github.rs](../../../shared/files/crates/hickory-cli/src/serve/github.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
