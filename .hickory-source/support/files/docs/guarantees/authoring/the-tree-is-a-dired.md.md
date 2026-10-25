<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/authoring/the-tree-is-a-dired.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `a615ed13a9eb0e804a97be868833e838c30a029b6ee22f761897ca9ec5bed1fe`.

<book:entry path="docs/guarantees/authoring/the-tree-is-a-dired.md">
# Filesystem Acts Keep Dired's Safety

Given an entry in the editable Files buffer, when a person opens its context
menu, then rename, move, copy, delete, new-folder, and new-file acts are routed
to one filesystem-operation endpoint inside the open folder. A rename never
overwrites, a move or copy names a destination, `.git` and paths outside the
root are refused, and the Git pane reports the resulting working-tree change.

Given an open file tab, when a person right-clicks its center tab or drawer
filename, then it offers the same file actions as its Files row, using the
same menu definition, action handlers, and operation prompts. The act targets
that file alone, independently of marked Files rows or the selected tab.
Document and generated-file tabs do not offer adoption. Unsaved drafts and
tool tabs do not acquire filesystem actions. These actions remain available
when the Files pane is closed.

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
opened on it. A renamed open `.md` room is not silently rebound and must be
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

File-tab menu verification (2026-10-17): `just test-file-tab-menu` covers menu
parity, delete confirmation and target, adoption, document/generated-file
restrictions, and center/drawer right-click forwarding. Shared handlers and
prompts are in `apps/web/src/shell/fileMenuActions.tsx`; tab resolution is in
`apps/web/src/views/useFileTabMenu.tsx`.

Issue 72 adapter extension (2026-10-19):

- useDired registers marks, operation preparation, current prompt/error, explicit
  confirmation/value submission and cancellation through its shared controller.
  Preparation does not execute a filesystem write. Submission awaits the actual
  controller result and retains host refusal text.
- TreeRenameEditor exposes its actual CodeMirror column, shared validation/apply
  handler and per-node outcomes. Successful nodes are retained across partial
  retries. Concurrent batch submission is refused.
- FolderRoot discloses immediate children in pages of at most 64 and opens a
  listed file through the same onOpen callback. File content is not implicitly
  included. Arbitrary/missing paths and unsupported binary views are refused.
- Executed checks: two focused dired adapter tests, two rename component tests
  and one folder disclosure/navigation test passed. These use injected host
  operations; no native filesystem or platform witness is inferred. Focused
  strict TypeScript checks passed for these controllers.
- Remaining scope: editable Files-buffer reconciliation, context menus, provider
  nodes and remaining tree gestures still need complete registered parity.


### Visible filesystem prompts — 2026-10-22

The actual prompt draft is read/editable through shared actions. Dired owns that
draft and its operation identity; parent submission displays its actual destination
while pending. Native typing invalidates old parent handles. Agent confirmation
requires explicit true. Synchronous owner/admission checks refuse stale callbacks,
duplicate submissions and prompt replacement/cancellation while the operation is
pending, including before React paints. A refused operation is not a success.
Partial multi-path failure retains the completed paths, refreshes the tree and
never guesses rollback; the failed prompt remains available for inspection.

LLM implementation review: partially verified for these paths. Eight focused
prompt/dired cases and the existing native delete-confirmation case passed;
targeted TypeScript passed. Host filesystem operations are injected, so this is
not a current native-platform witness. Dependencies, assumptions and remaining
issue-72 work are in the active handoff. Human judgment is not supplied.
</book:entry>

</book:doc>
