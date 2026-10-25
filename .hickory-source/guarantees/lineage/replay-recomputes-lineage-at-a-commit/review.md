<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Replay Recomputes Exact Lineage At Any Commit, And Names Its One Limit: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/lineage/TimeSlider.tsx](../../../shared/files/apps/web/src/lineage/TimeSlider.tsx.md)
- [apps/web/src/views/LineageView.tsx](../../../shared/files/apps/web/src/views/LineageView.tsx.md)
- [crates/hickory-cli/src/main.rs](../../../shared/files/crates/hickory-cli/src/main.rs.md)
- [crates/hickory-cli/src/replay.rs](files/crates/hickory-cli/src/replay.rs.md)
- [crates/hickory-cli/src/serve/history.rs](../../../shared/files/crates/hickory-cli/src/serve/history.rs.md)

## Tests and fixtures

- [apps/web/src/lineage/TimeSlider.test.tsx](files/apps/web/src/lineage/TimeSlider.test.tsx.md)
- [crates/hickory-cli/tests/replay_and_floor.rs](../../../shared/files/crates/hickory-cli/tests/replay_and_floor.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
