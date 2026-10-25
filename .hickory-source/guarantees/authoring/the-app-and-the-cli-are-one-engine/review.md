<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Desktop App And The CLI Are One Engine: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/desktop/src-tauri/src/server.rs](../../../shared/files/apps/desktop/src-tauri/src/server.rs.md)
- [crates/hickory-cli/src/serve/mod.rs](../../../shared/files/crates/hickory-cli/src/serve/mod.rs.md)

## Tests and fixtures

- [apps/desktop/src-tauri/tests/serves_one_origin.rs](../../../shared/files/apps/desktop/src-tauri/tests/serves_one_origin.rs.md)
- [crates/hickory-cli/tests/engine_lifecycle.rs](../../../shared/files/crates/hickory-cli/tests/engine_lifecycle.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
