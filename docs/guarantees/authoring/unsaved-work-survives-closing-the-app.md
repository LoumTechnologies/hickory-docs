# Unsaved Work Survives Closing The App

Given a buffer that differs from its last explicit Save, the tab and the same
file in the Files pane carry an asterisk. When its tab or the window closes,
the app asks whether to save and says in that question whether closing without
saving will retain the changes. An unnamed document is discarded if it is not
saved, so every new Untitled document starts blank. A file that has been saved
before is retained only when **Settings → Editing → Retain unsaved changes**
is on; that setting is off by default.

When retained work is opened again, the buffer holds it, still unsaved. A
crash-recovery write and an explicit Save are different facts: making bytes
durable must not silently remove the asterisk or answer the Save decision for
the person.

Closing a window is not an implicit decision either to save or to throw away
previously named work. The prompt is the decision, and its wording makes
recovery visible rather than asking from the false premise that “not saved”
means “lost.” An unnamed document is the stated exception: it must be saved
to become a document, or it is discarded.

Four rules make it true rather than mostly true:

1. **A saved file's draft is written while you type, not at shutdown.** A draft written
   only on the way out is a draft that is not there after the one event most
   likely to lose work. A slow timer writes the buffer whenever it has moved,
   and `pagehide` flushes whatever the timer has not reached yet.
2. **The buffer is read through a callback, never a prop.** An editor pane
   deliberately does not re-render per keystroke — CodeMirror owns the text —
   so there is no prop that changes when the buffer does, and inventing one
   would put a React render on the typing path to power a background save.
3. **Every draft records the bytes it was taken from.** That base is the
   common ancestor of the buffer and whatever the file says now, and it is
   what makes the reopen safe rather than merely possible.
4. **Nothing is written inside the project.** The store lives under the user's
   own data directory, keyed by the project's canonical path, so git cannot
   reach it by construction. A `.gitignore` entry would be a promise this tool
   cannot keep on somebody else's machine — a `git add -A` in a folder where
   `hick init` never ran, a fresh clone, a rewritten ignore file — and what
   would leak is unfinished work its author has not decided to keep.

## What happens when the file moved on

Three outcomes, and only one of them interrupts anybody:

- **The file is as we left it.** The text goes back into the buffer, still
  unsaved, silently. This is the common case by a wide margin, and a dialog
  here would train people to dismiss dialogs.
- **The file already says the same thing.** Somebody saved that text from
  somewhere else. The draft is stale and is discarded — and this is checked
  *first*, so a draft matching the file is never treated as a conflict however
  far the file has moved from the base.
- **The file moved on and so did the buffer.** A merge, and it is worth
  someone's attention.

The merge is diff3, over the recorded base, which means most of it is not a
question: a region only one side changed is that side's edit and is taken
without asking. What is left is shown as a conflict. Auto-resolved regions are
shown too, labelled with who they came from — a merge tool that silently takes
a side is a merge tool nobody trusts twice — and the summary leads with how
much of it is actually the reader's problem, because "11 merged, 2 need you"
is the difference between a merge someone answers and one they abandon.

Accepting with conflicts still unanswered is allowed, and the button says what
it will do. An unanswered conflict already reads as the reader's own text —
which is what the buffer holds at that moment — so accepting is a real choice
("everything I did not answer stays mine"), and blocking it would trap someone
in a dialog over a region they do not care about.

The same merge is offered from the save-conflict banner, where the two
existing answers each throw away one side's work.

## Boundary

**Nothing here is required for the app to run.** A store that cannot be opened
— no data directory, a read-only home — degrades to "the window forgets", says
so on the route, and never blocks a start. `HICKORY_STATE_DIR` names the
directory for a portable install.

This covers `.hick` document buffers. An unnamed document is deliberately not
recovered: it has no identity until Save names it, and a fresh document must be
blank. A `.hick` room may persist live bytes for crash recovery while the
explicit-Save baseline stays put; those are still unsaved changes in the
user-facing sense. Plain files retain their existing short debounced-save
window and draft machinery.

A draft is keyed by path. Renaming a file outside the app orphans its draft
rather than following it — the draft is still there, under the old path, and
will be offered if a file reappears at that path.

Two-way merge exists for the case with no ancestor at all (a buffer that never
had a file behind it, where a file now exists). It is honest about being
worse: every differing run is a conflict, because without a base there is no
way to tell an addition on one side from a deletion on the other, and a tool
that guessed would silently throw away work.


> **Amended 2026-09-02.** Found by dogfooding on this repository: switching
> away from a plain file's tab **emptied the file on disk**. The draft
> keeper's final flush runs during unmount, after the effect that destroys
> the editor view (React runs cleanups in declaration order), and reading the
> buffer through the dead view gave `""` — recorded as a draft of nothing,
> restored "silently" on the next mount because the disk still matched the
> base, and then autosaved. The pane now keeps the buffer's last text in a
> ref and reads that when the view is gone; an absent view is not an empty
> buffer. `PlainFilePane.test.tsx` ("closing the pane") fails on the old code.

---

Last LLM verification:

- Date: 2026-09-16
- Reviewer: Codex (GPT-5)
- Result: partially verified
- Evidence: `apps/web/src/views/documentSession.tsx` — explicit-Save baseline,
  dirty state, recovery draft restore, save and discard;
  `useUnsavedLifecycle.ts` — the tab/window close decisions and dirty path/id
  derivation; `WorkspaceView.tsx` — the shared wiring;
  `workspaceTabs.tsx` — blank Untitled buffer; `ShellView.tsx` and
  `FolderTreePane.tsx` — the two asterisk surfaces; `SettingsView.tsx`,
  `UiStore`, and `/api/settings/ui` — the off-by-default setting;
  `apps/desktop/src-tauri/src/lib.rs` and `serve/shell.rs` — native close
  interception and the approved-close handoff.
- Test coverage: `useUnsavedLifecycle.test.tsx` (unnamed-document discard and
  guarded close), `ShellView.test.tsx` (asterisk and guarded close),
  `workspaceTabs.test.tsx` (a new Untitled buffer never restores old text),
  `FolderTreePane.test.tsx` (file-tree asterisk), `SettingsView.test.tsx`
  (default and opt-in), and `serve_ui_settings.rs` (persisted schema).
- Caveat requiring review: a browser-level test does not yet drive the whole
  native window-close handshake, and the pre-existing plain-file autosave path
  is not converted to an explicit-Save baseline by this guarantee.
