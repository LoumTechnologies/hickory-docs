<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Merged View Is A Lens Over Several Worktrees, Never A Document: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/views/MergedView.tsx](../../../shared/files/apps/web/src/views/MergedView.tsx.md)
- [crates/hick-merge/src/nway.rs](../../../shared/files/crates/hick-merge/src/nway.rs.md)
- [crates/hickory-cli/src/serve/merged.rs](../../../shared/files/crates/hickory-cli/src/serve/merged.rs.md)

## Tests and fixtures

- [apps/web/src/views/MergedView.test.tsx](files/apps/web/src/views/MergedView.test.tsx.md)
- [crates/hickory-cli/tests/merged_view.rs](../../../shared/files/crates/hickory-cli/tests/merged_view.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
