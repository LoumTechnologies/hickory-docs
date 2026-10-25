<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Candidate revision checks never borrow working bytes: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hickory-cli/src/init_hook.sh](../../../support/files/crates/hickory-cli/src/init_hook.sh.md)
- [crates/hickory-cli/src/project_command.rs](../../../support/files/crates/hickory-cli/src/project_command.rs.md)
- [crates/hickory-cli/src/project_index.rs](../../../support/files/crates/hickory-cli/src/project_index.rs.md)
- [crates/hickory-cli/src/project_revision.rs](../../../shared/files/crates/hickory-cli/src/project_revision.rs.md)

## Tests and fixtures

- [crates/hickory-cli/tests/project_revision.rs](../../../shared/files/crates/hickory-cli/tests/project_revision.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
