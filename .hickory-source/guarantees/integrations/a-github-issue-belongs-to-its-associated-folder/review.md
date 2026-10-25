<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A GitHub Issue Belongs To Its Associated Folder: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/shell/FolderTreePane.tsx](../../../shared/files/apps/web/src/shell/FolderTreePane.tsx.md)
- [apps/web/src/shell/GithubTreeNodes.tsx](../../../shared/files/apps/web/src/shell/GithubTreeNodes.tsx.md)
- [apps/web/src/shell/treeMenu.ts](../../../shared/files/apps/web/src/shell/treeMenu.ts.md)
- [crates/hickory-cli/src/serve/github.rs](../../../shared/files/crates/hickory-cli/src/serve/github.rs.md)

## Tests and fixtures

- [apps/web/src/shell/FolderTreePane.test.tsx](../../../shared/files/apps/web/src/shell/FolderTreePane.test.tsx.md)
- [apps/web/src/shell/GithubTreeNodes.test.tsx](../a-review-belongs-to-the-worktree-for-its-head/files/apps/web/src/shell/GithubTreeNodes.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
