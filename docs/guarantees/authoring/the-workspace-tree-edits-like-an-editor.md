# The Workspace Tree Is An Editor Buffer

Given an open folder, when the Files pane is shown, then the filesystem is
written into an ordinary CodeMirror buffer: one entry per line, a trailing
slash distinguishes a folder, and exactly two leading spaces per level place
an entry inside the folder above it. The bytes are the tree. They are not
labels laid over buttons and there is no separate edit mode.

Given that buffer, when a person clicks, moves with the arrow keys, selects
text, uses multiple cursors or rectangular selection, types, or undoes, then
the same editor mechanisms used in a `.md` pane answer those gestures. A
single click places the caret. Double-clicking a file selects exactly its whole
line and opens it, replacing the editor's usual word selection with the Files
buffer's activation gesture. The fold gutter may hide descendants without
removing their text. Show Files only focuses the
buffer; focusing it is never required before its text can be edited.

Given unchanged filesystem lines are reordered, then their order is
presentation only: the buffer remains clean and neither Dry Run nor Apply
reports a filesystem operation. Hickory matches unchanged root-relative paths
and file/folder kinds before interpreting the remaining name or indentation
edits. Reordering plus a real rename reports only that rename.

Given existing filesystem lines whose names or significant indentation were
edited, when the buffer is saved, then Hickory validates the whole reading and
applies the corresponding root-relative rename or move operations parent-first.
A renamed folder carries unchanged descendants without sending false child
renames. Refusals are written beneath the buffer, the filesystem is refreshed,
and Hickory never claims that an edit was applied when the server refused it.

Given the buffer differs from its projection, then a toolbar appears across
its top with Dry Run and Apply. Dry Run uses the same prepared plan as Apply,
performs no writes, and lists every operation in execution order with its full
root-relative source and destination or target value. An invalid buffer shows
the exact refusal there. Apply executes that plan; `Ctrl+S`/`Cmd+S` remains a
shortcut for Apply. The toolbar is absent when the buffer is clean and its
actions are disabled while an apply is running.

Given new lines, saving creates the named files and trailing-slash folders in
parent-first order. A new folder plus an existing file, unchanged except for
indentation beneath it, is one unambiguous save: Hickory creates the folder,
then moves the file. A new entry combined with renaming an existing entry is
still refused, because the text contains no hidden identity that can tell a
rename from a create. Given removed lines, saving stages the topmost removed
paths and asks for confirmation because there is no trash; only the explicit
Delete applies them. Removing a folder and its visible descendants is one
reviewed directory delete, not a stream of already-redundant child deletes.

Terminal sessions and associated GitHub issues are projected as indented text
beneath their folders. Activating one routes to that object's own action. An
edit confined to an issue's title calls its GitHub title capability; edits to
its immutable number or presentation state are refused visibly. Terminal text
remains read-only until terminal-title persistence exists.

## Boundary

To keep identity understandable without hidden ids in the text, create/delete
is separate from a rename, except for the unambiguous move of an unchanged,
uniquely named entry into a newly created folder. A remote title edit is saved
separately from deletion. A buffer currently showing live terminal or issue
lines refuses filesystem line-count changes until those projected lines can be
tracked through arbitrary edits. Remote-object body and comment editing still
needs expandable editor regions; the editor must never pretend a display-only
edit reached GitHub or Jira.

---

Last LLM verification:
- Date: 2026-09-15
- Reviewer: Codex
- Result: partially verified — the literal buffer, editor gestures, multiple
  and rectangular selections, order-insensitive rows, name/indent rename and
  move, create, reviewed delete, folding, file
  activation, terminal/issue placement, GitHub issue-title edit, dirty-only
  Apply/Dry Run toolbar, exact non-mutating plan, visible refusal, and live
  filesystem round trip are built; projected-line tracking and remote
  body/comment regions remain at the boundary
- Evidence: `apps/web/src/shell/FilesystemTreeEditor.tsx`,
  `FolderTreePane.tsx`, `apps/web/src/lib/filesystemTreeText.ts`, and
  `apps/web/src/editor/multiCursor.ts`
- Test coverage: `filesystemTreeText.test.ts`,
  `FilesystemTreeEditor.test.tsx`, `FolderTreePane.test.tsx` (literal bytes,
  focus, whole-line double-click activation, multiple selection,
  order-insensitive rows, dirty toolbar, exact non-mutating dry runs,
  rename/move/create/delete, activation, and associated objects), and
  `apps/web/e2e/files-editor.spec.ts`, which verifies Dry Run leaves the live
  filesystem unchanged before applying real rename, create, and confirmed
  delete operations against `just dev`

## Shared Files reconciliation controls (2026-10-20)

The Files buffer opts into the same bounded editor source, selection, fold,
input and undo/redo controllers as document editors. Its own Dry Run, Apply,
confirm deletion and cancel deletion commands share the native handlers; status,
preview, required deletion request and operation receipts have explicit paged
reading. Associated-object title writes retain their existing semantic callback.

A deletion confirmation binds the current buffer bytes and projection version;
changing the draft invalidates it. Apply is serialized by an immediate controller
lock and refuses native document changes while pending. Each operation boundary
checks the captured editor generation, projection version and cancellation signal.
Cancellation cannot roll back an already dispatched filesystem/provider request.
Actual completed responses and the stopped/failed operation are reported; later
operations do not start after refusal/cancellation/owner replacement. A failed
request can itself have uncertain effects. This is not an atomic filesystem batch,
external-edit CAS, or proof that a remote title changed beyond its callback reply.

Focused evidence: `FilesystemTreeEditor.test.tsx` covers scripted Python/ACP actor
paths, native input during a pending request, duplicate submission refusal,
non-mutating preview, exact host replies, invalidated deletion confirmation,
cancellation after the first request, partial failure, existing native deletion
and semantic provider title application. Six selected cases passed together;
strict focused TypeScript, inventory, file-length and diff checks also passed.
These are mocked host replies with real CodeMirror, not a native/provider witness.
