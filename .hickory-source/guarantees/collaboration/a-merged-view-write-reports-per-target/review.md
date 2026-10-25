<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Write Through The Merged View Reports Per Target, And Undo Reaches All Of Them: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-merge/src/nway.rs](../../../shared/files/crates/hick-merge/src/nway.rs.md)
- [crates/hickory-cli/src/serve/merged.rs](../../../shared/files/crates/hickory-cli/src/serve/merged.rs.md)
- [crates/hickory-cli/src/serve/merged_write.rs](../../../support/files/crates/hickory-cli/src/serve/merged_write.rs.md)

## Tests and fixtures

- [crates/hickory-cli/tests/merged_view.rs](../../../shared/files/crates/hickory-cli/tests/merged_view.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
