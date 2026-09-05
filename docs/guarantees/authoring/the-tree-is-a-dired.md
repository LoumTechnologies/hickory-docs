# The Tree Is A Dired

Given the file tree of an open folder, when a person marks rows and acts on
them — Ctrl+click (Cmd on a Mac) toggles a mark, and with a row focused
`m` marks, `u` unmarks, `U` unmarks all, `D` or Delete deletes, `R`
renames, `C` copies, `M` moves, `+` makes a folder and `n` a file — or picks
the same verbs from a row's right-click menu, then each verb is one
filesystem operation on the server inside the open folder: a rename never
overwrites, a move or copy goes into a named directory, a delete removes
from the disk and asks once first because there is no trash, and a new
file or folder is made where the row is. A verb on a marked row acts on
every mark; on an unmarked row it acts on that row alone, the way dired
does. The tree refetches after each, and the marks clear.

Emacs's dired is the reason for the keys: a person who wants dired wants
those keys, and nobody else is hurt by them — they fire only while a tree
row has the focus and never with a modifier held.

Three properties hold it up:

1. **The keys are data.** `shell/dired.ts` turns a key, the focused row
   and the marks into an intent, and `useDired` runs intents: mark
   intents change the set, verbs open one prompt (`TreePrompt`, in the
   pane — never a browser `prompt()`, which the desktop shell would block
   on) and then call `POST /api/files/op` once per target.
2. **One route, six verbs, the refusals said plainly.**
   `serve/files_ops.rs`: nothing outside the folder or under `.git`, never
   the folder itself, never over something that exists (`409`), a move into
   itself refused, a `.hick` "file" pointed at New Document, an unknown verb
   named. No `git mv`: the Git pane shows what changed.
3. **The menu and the keys agree.** `treeMenuItems` builds the verbs from
   the same marks (`subjects`), so "Delete (3 marked)" in the menu and `D`
   on the row do the same thing.

## Boundary

No drag-and-drop between rows, no undo: a delete is final, which is why it
asks. A rename of a `.hick` document is a rename on disk; the open room, if
any, is not moved with it and must be reopened. The root row itself cannot
be renamed, moved or deleted from inside the app that is open on it.

---

Last LLM verification:
- Date: 2026-09-05
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `apps/web/src/shell/dired.ts`, `useDired.ts`, `TreePrompt.tsx`,
  `treeMenu.ts` (`subjects`, the verb items), `FolderTreePane.tsx` (marks
  on rows, Ctrl+click, the tree's `onKeyDown`, `runItem`);
  `crates/hickory-cli/src/serve/files_ops.rs` routed as `POST /files/op`.
- Test coverage: `apps/web/src/shell/dired.test.ts`,
  `treeMenu.test.ts` ("the dired and ingest verbs"),
  `FolderTreePane.test.tsx` ("the tree as dired");
  `crates/hickory-cli/tests/tree_file_ops.rs` over real HTTP, every verb
  and every refusal.
