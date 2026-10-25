<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# `.md` Documents Merge Through Hick, And A Clone That Cannot Is Told: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/MergeDriverNotice.tsx](../../../shared/files/apps/web/src/components/MergeDriverNotice.tsx.md)
- [crates/hickory-cli/src/init.rs](../../../shared/files/crates/hickory-cli/src/init.rs.md)
- [crates/hickory-cli/src/main.rs](../../../shared/files/crates/hickory-cli/src/main.rs.md)
- [crates/hickory-cli/src/merge_driver.rs](../../../shared/files/crates/hickory-cli/src/merge_driver.rs.md)
- [crates/hickory-cli/src/serve/history.rs](../../../shared/files/crates/hickory-cli/src/serve/history.rs.md)

## Tests and fixtures

- [apps/web/src/components/MergeDriverNotice.test.tsx](../../../shared/files/apps/web/src/components/MergeDriverNotice.test.tsx.md)
- [crates/hickory-cli/tests/replay_and_floor.rs](../../../shared/files/crates/hickory-cli/tests/replay_and_floor.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
