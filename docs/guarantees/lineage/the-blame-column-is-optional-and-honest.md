# The Blame Column Is Optional And Honest

Given any editor in the app, when the blame column is off — which it is by
default — then nothing is asked of git at all; and when it is turned on, then
every line shows who last committed it, uncommitted lines say so rather than
borrowing an author, and a folder with no history shows an empty column rather
than failing to open a file.

## Why off by default

A blame column is a permanent indent on every line of every file, answering a
question nobody asks most of the time. It earns its place when you *are*
asking it — reviewing, bisecting, or working out whether a passage was written
by a person or by the agent — and the rest of the time it is less room for the
code.

Off by default is only honest if it is also free, so the fetch is lazy:
nothing is asked of git until somebody turns the column on. A `git blame` per
open file at startup would cost real time on a large repository to fill a
column nobody is looking at.

## The rules

1. **One `git blame` per file, not per line.** The single-span
   `agent_lineage::blame` shells out per call, which is right for a provenance
   question asked once. A column asks about every line at once, and doing that
   a line at a time would fork a process per line.
2. **Left of the numbers.** The line numbers are the coordinate everything
   else in this app refers to — a stack trace, a ribbon, a collaborator saying
   "line 40" — so they stay against the text and the annotation goes outside
   them. CodeMirror lays gutters out in declaration order, which is what
   `blameGutter()` before `lineNumbers()` means.
3. **A run of lines from one commit is labelled once**, at its first line, and
   left blank below. Repeating the same name and date down forty rows is how a
   blame column becomes wallpaper: the eye stops reading it, and the
   boundaries — which are the actual information — disappear into the
   repetition.
4. **Uncommitted is not attributed to anybody else.** The working tree is the
   common case, not an edge case. A line somebody just typed reads "you —
   uncommitted"; attributing it to whoever last touched the file would be a
   lie in the direction that matters most.
5. **No history is not an error.** Not a repository, untracked, no git on the
   machine: an empty column, and the file opens exactly as it would have.

## Boundary

The column refills when files change on disk (a save, a run, a checkout), not
as you type. Blame describes what is committed; re-running it per keystroke
would burn a process per character to move a label that has not changed. The
consequence is that the column is briefly stale while you type, which is
correct — the line is not committed yet, and the row above it still says who
committed the line above it.

The toggle is per machine (localStorage), not per project or per tab: it
follows the job the reader is doing, and two panes disagreeing about whether
the column is up would look like a bug.

This does **not** yet mark AI-written spans distinctly. The composition that
answers that question — lineage from `SourceOrigin::Agent` to a document span,
then blame on the span — lives in `agent_lineage.rs` and is not wired into
this column. Showing "author" is what this guarantees; showing "human or
model" is a further claim, and `provenance-and-standing.md` is explicit that
the two must never render alike.

Dates are relative under a year (`3d ago`, `2mo ago`) and a year past that,
because "412 days ago" is not something anyone converts in their head.

---

Last LLM verification:
- Date: 2026-08-21
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `crates/hickory-cli/src/agent_lineage.rs::blame_file` — the
  whole-file porcelain parse, including the commit-details cache porcelain
  requires (a commit's author is named once and referred to by sha after).
  `crates/hickory-cli/src/serve/api.rs::blame` — the route, its path
  bounds-check, and `spawn_blocking` (git is process work).
  `apps/web/src/editor/blameGutter.ts` — `blameField` (off by default),
  `rowLabel` (the run-labelling rule), `blameDate`, `shortAuthor`, `blameTip`,
  and the spacer's own class so nothing counts it as an entry.
  `apps/web/src/editor/useBlame.ts` — the lazy fetch, the toggle listener, and
  the refill on `FILES_CHANGED_EVENT`.
  `apps/web/src/lib/blamePref.ts` — the per-machine preference and the event
  that turns every editor together.
  Wired before `lineNumbers()` in `PlainFilePane`, `OutputEditorPane`, and
  (before the debug layer, which is where the numbers come from) in
  `DocumentEditor`. Menu: `blame` in `menuBridge.ts`, the View submenu in
  `apps/desktop/src-tauri/src/lib.rs`, handled in `views/useZoom.ts`.
- Test coverage: `crates/hickory-cli/tests/serve_blame.rs` (5 tests) — every
  line attributed from one blame, the uncommitted case not borrowing an
  author, a non-repository answering `[]`, an untracked file answering `[]`,
  and a path outside the folder refused.
  `apps/web/src/editor/blameGutter.test.ts` (18 tests) — the date scale
  including the year cutoff and a missing time, author fitting and
  truncation, the run-labelling rule in both directions (including that an
  uncommitted line does not run into a committed one above it), the hover
  content, and the column in a real `EditorView`: nothing drawn until it is
  on, rows once it is, gone again when it is off, and `data-tip` rather than
  a native `title`.
- Caveat requiring review: the column's WIDTH is not asserted — jsdom does no
  layout, so "does a 14em cap actually leave room for the code" was checked by
  eye. The refill-on-save path is wired but not tested end to end; the fetch
  and the event are each covered separately.
