# Filesystem Acts Keep Dired's Safety

Given an entry in the editable Files buffer, when a person opens its context
menu, then rename, move, copy, delete, new-folder, and new-file acts are routed
to one filesystem-operation endpoint inside the open folder. A rename never
overwrites, a move or copy names a destination, `.git` and paths outside the
root are refused, and the Git pane reports the resulting working-tree change.

Editing a line's name or indentation and saving is the ordinary rename/move
path. Inserting a line creates its file or trailing-slash folder. Removing
lines stages a destructive delete and asks once before removing bytes from
disk because there is no trash. The context menu offers the same reviewed
delete. Provider and terminal lines do not acquire filesystem verbs merely
because they are drawn inside the same editor.

The server is deliberately independent of the presentation. `POST
/api/files/op` answers `rename`, `move`, `copy`, `delete`, `mkdir`, and
`create`; each refusal says what was unsafe or stale. The pane refreshes after
an act, and never guesses an inverse operation after a partial failure.

## Boundary

Create/delete and rename/move are separate saves so line identity is never
guessed. The root itself cannot be renamed, moved, or deleted from the app
opened on it. A renamed open `.hick` room is not silently rebound and must be
reopened.

---

Last LLM verification:
- Date: 2026-09-15
- Reviewer: Codex
- Result: verified for context-menu acts, server safety, direct text rename,
  move and create, and text/context-menu destructive confirmation
- Evidence: `apps/web/src/shell/FilesystemTreeEditor.tsx`, `treeMenu.ts`,
  `useDired.ts`, `TreePrompt.tsx`, and
  `crates/hickory-cli/src/serve/files_ops.rs`
- Test coverage: `filesystemTreeText.test.ts`,
  `FilesystemTreeEditor.test.tsx`, `treeMenu.test.ts`,
  `FolderTreePane.test.tsx` ("the tree as dired"),
  `crates/hickory-cli/tests/tree_file_ops.rs`, and the live Playwright Files
  buffer rename flow
