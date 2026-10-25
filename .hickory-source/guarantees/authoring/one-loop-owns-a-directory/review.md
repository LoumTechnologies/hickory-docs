<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# One loop owns a directory and never reacts to itself: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hickory-cli/src/engine/mod.rs](../../../shared/files/crates/hickory-cli/src/engine/mod.rs.md)
- [crates/hickory-cli/src/up/mod.rs](../../../shared/files/crates/hickory-cli/src/up/mod.rs.md)
- [crates/hickory-cli/src/up/state.rs](../../../shared/files/crates/hickory-cli/src/up/state.rs.md)

## Tests and fixtures

- [apps/desktop/src-tauri/tests/serves_one_origin.rs](../../../shared/files/apps/desktop/src-tauri/tests/serves_one_origin.rs.md)
- [crates/hickory-cli/tests/engine_lifecycle.rs](../../../shared/files/crates/hickory-cli/tests/engine_lifecycle.rs.md)
- [crates/hickory-cli/tests/literate_views.rs](../../../support/files/crates/hickory-cli/tests/literate_views.rs.md)
- [crates/hickory-cli/tests/up_loop.rs](../../../shared/files/crates/hickory-cli/tests/up_loop.rs.md)
- [crates/hickory-cli/tests/up_stress.rs](../../../shared/files/crates/hickory-cli/tests/up_stress.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</book:doc>
