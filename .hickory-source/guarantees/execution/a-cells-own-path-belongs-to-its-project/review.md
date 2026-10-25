<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Cell's Own Path Belongs To Its Project, Not To Your Shell: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hickory-cli/src/lib.rs](../../../shared/files/crates/hickory-cli/src/lib.rs.md)
- [crates/hickory-executor-sandbox/src/lib.rs](../../../shared/files/crates/hickory-executor-sandbox/src/lib.rs.md)
- [crates/hickory-executor/src/lib.rs](../../../shared/files/crates/hickory-executor/src/lib.rs.md)

## Tests and fixtures

- [crates/hickory-cli/tests/cache_inputs.rs](../../../shared/files/crates/hickory-cli/tests/cache_inputs.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
