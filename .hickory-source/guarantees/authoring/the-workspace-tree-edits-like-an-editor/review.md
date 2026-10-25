<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Workspace Tree Is An Editor Buffer: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/editor/multiCursor.ts](../../../shared/files/apps/web/src/editor/multiCursor.ts.md)
- [apps/web/src/lib/filesystemTreeText.ts](files/apps/web/src/lib/filesystemTreeText.ts.md)
- [apps/web/src/shell/FilesystemTreeEditor.tsx](../../../shared/files/apps/web/src/shell/FilesystemTreeEditor.tsx.md)
- [apps/web/src/shell/FolderTreePane.tsx](../../../shared/files/apps/web/src/shell/FolderTreePane.tsx.md)

## Tests and fixtures

- [apps/web/e2e/files-editor.spec.ts](files/apps/web/e2e/files-editor.spec.ts.md)
- [apps/web/src/lib/filesystemTreeText.test.ts](files/apps/web/src/lib/filesystemTreeText.test.ts.md)
- [apps/web/src/shell/FilesystemTreeEditor.test.tsx](files/apps/web/src/shell/FilesystemTreeEditor.test.tsx.md)
- [apps/web/src/shell/FolderTreePane.test.tsx](../../../shared/files/apps/web/src/shell/FolderTreePane.test.tsx.md)
- [apps/web/src/shell/GithubBodyAndComment.test.tsx](../../../support/files/apps/web/src/shell/GithubBodyAndComment.test.tsx.md)
- [apps/web/src/shell/GithubTreeControls.test.tsx](../../../support/files/apps/web/src/shell/GithubTreeControls.test.tsx.md)
- [apps/web/src/shell/GithubTreeNodes.test.tsx](../../integrations/a-review-belongs-to-the-worktree-for-its-head/files/apps/web/src/shell/GithubTreeNodes.test.tsx.md)
- [apps/web/src/shell/TreeRenameEditor.test.tsx](files/apps/web/src/shell/TreeRenameEditor.test.tsx.md)
- [apps/web/src/shell/treeNavigation.test.ts](files/apps/web/src/shell/treeNavigation.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
