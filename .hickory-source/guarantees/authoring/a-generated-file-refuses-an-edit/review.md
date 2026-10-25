<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Generated Text Refuses An Edit, And Says So Before You Type: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hickory-cli/src/lib.rs](../../../shared/files/crates/hickory-cli/src/lib.rs.md)
- [crates/hickory-cli/src/serve/plain_file.rs](../../../shared/files/crates/hickory-cli/src/serve/plain_file.rs.md)
- [crates/hickory-cli/src/up/mod.rs](../../../shared/files/crates/hickory-cli/src/up/mod.rs.md)
- [crates/hickory-cli/src/up/reverse.rs](../../../shared/files/crates/hickory-cli/src/up/reverse.rs.md)
- [crates/hickory-cli/src/up/state.rs](../../../shared/files/crates/hickory-cli/src/up/state.rs.md)

## Tests and fixtures

- [crates/hickory-cli/tests/up_loop.rs](../../../shared/files/crates/hickory-cli/tests/up_loop.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
