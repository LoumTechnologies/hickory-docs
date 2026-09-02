# Unsaved Work Survives Closing The App

Given a buffer holding text the file on disk does not have, when the app is
closed — gracefully, by a crash, or by the operating system discarding the
window — then that text is written down; and when the app is opened again, the
buffer holds it, still unsaved, in the tab it was left in.

Closing a window is not a decision to throw work away. An editor that treats
it as one teaches people to be afraid of ⌘W, and the fear is the damage: it
makes every close a small negotiation instead of a reflex.

Four rules make it true rather than mostly true:

1. **The draft is written while you type, not at shutdown.** A draft written
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

This covers **plain files**, which are the buffers that can genuinely hold
unsaved text: a `.hick` document is a CRDT room that persists as you type, and
a woven output saves through its document. If those ever grow an unsaved
state, they use this same store; today they have none to keep.

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
- Date: 2026-08-20
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `crates/hickory-workspace/src/lib.rs` — the store, its header
  explaining why not the project folder, `write_atomically` (temp + fsync +
  rename, because "survives the app closing" includes closing badly), the
  size caps, and `STATE_DIR_VAR`.
  `crates/hickory-cli/src/serve/workspace.rs` — five routes, and `store()`
  mapping a store that will not open to a 503 that says the app still works.
  `apps/web/src/lib/drafts.ts` — `draftDisposition` (the three outcomes, in
  the order that matters) and `useDraftKeeper` (poll + `pagehide`, read
  through a callback).
  `apps/web/src/lib/merge.ts` — diff3: `mergeThreeWay`, `mergeTwoWay`,
  `mergedText` (unanswered conflicts read as ours), `matchedLines`.
  `apps/web/src/components/MergeView.tsx` — the summary-first layout,
  auto-resolved regions labelled rather than hidden, accept-with-unanswered.
  `apps/web/src/components/PlainFilePane.tsx` — the load-time draft check,
  `putInBuffer`, `useDraftKeeper`, and `openMerge`/`acceptMerge` including the
  new third answer on the conflict banner.
  `apps/web/src/lib/plainFileSave.ts` — `baseContent()`, the ancestor.
- Test coverage: `crates/hickory-workspace/src/lib.rs` (11 tests) — including
  `nothing_is_written_inside_the_project`, `a_partial_write_never_replaces_a_good_draft`,
  `two_projects_with_the_same_name_do_not_share_a_store`, and the
  damaged-layout and oversized-blob refusals.
  `crates/hickory-cli/tests/serve_workspace.rs` (6 tests) — the routes over
  real HTTP, including `nothing_is_written_inside_the_project_folder`.
  `apps/web/src/lib/merge.test.ts` (41 tests) — the diff3 laws asserted over
  seven awkward shapes (empty files, no trailing newline, whole-file deletion,
  append plus prepend), what it resolves alone, what it must ask about, and
  the two-way fallback.
  `apps/web/src/lib/drafts.test.ts` (5 tests) — the disposition decision,
  including a deleted file and a draft with no base.
  `apps/web/src/components/MergeView.test.tsx` (11 tests) — the summary
  count, auto-merged regions named, both sides labelled, accept with and
  without answers, un-answering, and the two-way variant hiding "keep
  neither".
  `apps/web/src/components/PlainFilePane.test.tsx` (5 tests) — restore
  quietly, discard a stale draft, open a merge, no draft, and an unreachable
  store.
- Caveat requiring review: the crash path is argued rather than tested — the
  poll interval and the `pagehide` listener are asserted to exist by the
  hook's shape, but no test kills a process mid-edit and reopens it. The
  end-to-end "close the app, reopen it, the text is there" journey is likewise
  not driven by a test; the two halves (the store round-trips over HTTP, the
  pane restores from what that store answers) are each covered, and the join
  between them was checked by hand.
