<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Filesystem Acts Keep Dired's Safety: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/api/files.ts](files/apps/web/src/api/files.ts.md)
- [apps/web/src/shell/FilesystemTreeEditor.tsx](../../../shared/files/apps/web/src/shell/FilesystemTreeEditor.tsx.md)
- [apps/web/src/shell/TreePrompt.tsx](../../../support/files/apps/web/src/shell/TreePrompt.tsx.md)
- [apps/web/src/shell/dired.ts](files/apps/web/src/shell/dired.ts.md)
- [apps/web/src/shell/fileMenuActions.tsx](files/apps/web/src/shell/fileMenuActions.tsx.md)
- [apps/web/src/shell/treeMenu.ts](../../../shared/files/apps/web/src/shell/treeMenu.ts.md)
- [apps/web/src/shell/useDired.ts](files/apps/web/src/shell/useDired.ts.md)
- [apps/web/src/views/useFileTabMenu.tsx](files/apps/web/src/views/useFileTabMenu.tsx.md)
- [crates/hickory-cli/src/serve/files_ops.rs](files/crates/hickory-cli/src/serve/files_ops.rs.md)

## Tests and fixtures

- [apps/web/src/lib/filesystemTreeText.test.ts](../the-workspace-tree-edits-like-an-editor/files/apps/web/src/lib/filesystemTreeText.test.ts.md)
- [apps/web/src/shell/FilesystemTreeEditor.test.tsx](../the-workspace-tree-edits-like-an-editor/files/apps/web/src/shell/FilesystemTreeEditor.test.tsx.md)
- [apps/web/src/shell/FolderTreePane.test.tsx](../../../shared/files/apps/web/src/shell/FolderTreePane.test.tsx.md)
- [apps/web/src/shell/dired.test.ts](files/apps/web/src/shell/dired.test.ts.md)
- [apps/web/src/shell/treeMenu.test.ts](../../../shared/files/apps/web/src/shell/treeMenu.test.ts.md)
- [crates/hickory-cli/tests/tree_file_ops.rs](files/crates/hickory-cli/tests/tree_file_ops.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
