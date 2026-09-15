# The Workspace Tree Is An Editor Buffer

Given an open folder, when the Files pane is shown, then the filesystem is
written into an ordinary CodeMirror buffer: one entry per line, a trailing
slash distinguishes a folder, and exactly two leading spaces per level place
an entry inside the folder above it. The bytes are the tree. They are not
labels laid over buttons and there is no separate edit mode.

Given that buffer, when a person clicks, moves with the arrow keys, selects
text, uses multiple cursors or rectangular selection, types, or undoes, then
the same editor mechanisms used in a `.hick` pane answer those gestures. A
single click places the caret. Double-clicking a file opens it. The fold gutter
may hide descendants without removing their text. Show Files only focuses the
buffer; focusing it is never required before its text can be edited.

Given existing filesystem lines whose names or significant indentation were
edited, when the buffer is saved, then Hickory validates the whole reading and
applies the corresponding root-relative rename or move operations parent-first.
A renamed folder carries unchanged descendants without sending false child
renames. Refusals are written beneath the buffer, the filesystem is refreshed,
and Hickory never claims that an edit was applied when the server refused it.

Given new lines, saving creates the named files and trailing-slash folders in
parent-first order. Given removed lines, saving stages the topmost removed
paths and asks for confirmation because there is no trash; only the explicit
Delete applies them. Removing a folder and its visible descendants is one
reviewed directory delete, not a stream of already-redundant child deletes.

Terminal sessions and associated GitHub issues are projected as indented text
beneath their folders. Activating one routes to that object's own action. An
edit confined to an issue's title calls its GitHub title capability; edits to
its immutable number or presentation state are refused visibly. Terminal text
remains read-only until terminal-title persistence exists.

## Boundary

To keep identity understandable without hidden ids in the text, create or
delete is saved separately from rename/move, and a remote title edit is saved
separately from deletion. A buffer currently showing live terminal or issue
lines refuses filesystem line-count changes until those projected lines can be
tracked through arbitrary edits. Remote-object body and comment editing still
needs expandable editor regions; the editor must never pretend a display-only
edit reached GitHub or Jira.

---

Last LLM verification:
- Date: 2026-09-15
- Reviewer: Codex
- Result: partially verified — the literal buffer, editor gestures,
  name/indent rename and move, create, reviewed delete, folding, file
  activation, terminal/issue placement, GitHub issue-title edit, visible
  refusal, and live filesystem round trip are built; projected-line tracking
  and remote body/comment regions remain at the boundary
- Evidence: `apps/web/src/shell/FilesystemTreeEditor.tsx`,
  `FolderTreePane.tsx`, `apps/web/src/lib/filesystemTreeText.ts`, and
  `apps/web/src/editor/multiCursor.ts`
- Test coverage: `filesystemTreeText.test.ts`,
  `FilesystemTreeEditor.test.tsx`, `FolderTreePane.test.tsx` (literal bytes,
  focus, ordinary selection, rename/move/create/delete, activation, and
  associated objects), and `apps/web/e2e/files-editor.spec.ts`, which edits
  the Files buffer and observes real on-disk rename, create, and confirmed
  delete against `just dev`
