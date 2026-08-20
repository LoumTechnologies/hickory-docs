# An Edit Saved In A Woven File Lands In Its Document

Given `hick up` running over a folder, when a generated output file is saved
with a change to bytes that came from a `.hick` document, then that change
appears in the document — byte-exactly, inside the block it came from — and
the output file is left holding the same bytes the user just typed.

This is what makes a `.hick` document editable with an editor that has never
heard of hick. The document stays the source of truth; the generated file is
a working surface onto it, and the surface is writable in both directions.

Four properties hold that up:

1. **The mapping is lineage, not a heuristic.** The edit is expressed as byte
   ranges in the output and mapped through the same `Provenance[]` that
   `hick lineage` prints and the editor's output pane uses
   (`crates/hickory-lineage/src/lib.rs`). There is no second, looser rule for
   the filesystem path — an edit either maps or it is refused.
2. **A save is diffed line by line.** Two regions edited between one save and
   the next arrive as two edits, not one span covering both. A single wide
   span would also cover whatever sits between them, and generated text
   between two edited regions would turn a legal save into a refusal.
3. **The diff is against the bytes we wrote, not a fresh weave.** Re-weaving
   to compare would fold in changes the user never made and attribute them to
   this save.
4. **The document is re-read before the edit is applied**, and the edit is
   refused if the document has moved since the weave it was computed against.
   This is the staleness rule the hashline anchors in `hick doc edit` already
   enforce, applied to the same problem arriving through the filesystem. **A
   refusal restores the file from the document**, because a refused edit that
   left the two sides disagreeing would strand them: the document is not
   dirty, so nothing re-weaves, and no further event ever arrives to reconcile
   them.
5. **A weave never overwrites an edit the loop has not consumed yet.** A weave
   can take seconds — a `--run` waits for real commands — so a save can arrive
   while one is in flight. Writing the freshly woven bytes over it would delete
   the typing *and* make the pending event look like an echo of the loop's own
   write, so the edit would vanish with no error at all. When the bytes on disk
   match neither the last write nor the new weave, the file is left alone and
   the pending event is processed as the edit it is.
6. **The loop watches before it weaves.** The first pass puts output files on
   disk early and can then take as long as the slowest cell, so starting the
   watcher afterwards would miss every edit made in that window. Events queue
   from the moment the loop starts and drain once the initial weave is done —
   late rather than lost. Under `--run` a fast weave-only pass runs first, so
   there is a recorded baseline for property 5 to compare against before any
   long execution begins.

The reverse direction holds too: an edit made in the document is woven out to
its files without being asked, so the two sides never sit disagreeing.

## Boundary

Whitespace the weave normalizes is not preserved: it maps back, gets woven
again, and comes out normalized. Binary outputs are not watched — they have no
text to diff and no lineage to map through.

An edit is applied to the document on disk, not to a running `hick serve`
session. Running both over one directory is prevented by the lock in
[`one-loop-owns-a-directory`](one-loop-owns-a-directory.md); making them
cooperate is open work.

---

Last LLM verification:
- Date: 2026-08-12
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-cli/src/up/reverse.rs` — `diff_to_edits` turns a
  save into `OutputEdit`s (grouping a Delete run with the Insert run that
  follows it, so a replacement is one edit rather than two spans);
  `source_edits_for_save` maps them through `hickory_lineage::map_edits`;
  `apply_to_documents` re-reads each document, compares it against the source
  recorded at the last weave, and bails rather than applying a stale edit.
  `crates/hickory-cli/src/up/mod.rs::handle_batch` consumes output saves before
  re-weaving documents, so one weave covers both directions, and restores the
  file when `apply_to_documents` refuses. `up/state.rs::write_output` returns
  early without writing when the bytes on disk match neither the last write nor
  the new weave (property 5), which is why `retain_outputs_of` prunes *after*
  writing rather than before — clearing the entries first would discard the
  comparison it depends on. `up/mod.rs::run` installs the watcher before the
  first weave and calls `establish_baselines` under `--run` (property 6).
- Test coverage: `crates/hickory-cli/tests/up_stress.rs` drives the loop under
  load: bursts of 20–30 saves with and without pauses, interleaved document and
  output edits, an edit typed while a two-second cell is running, a non-atomic
  truncate-and-dribble save, and ten documents edited at once. Its assertions
  are convergence properties polled to a deadline rather than timings — with one
  exception, corrected 2026-08-19 and described below, where the test was
  arithmetic-dependent and did flake on a loaded runner. The "two-second cell"
  is written per shell (`tests/common/mod.rs::sleeps_then_echoes`): `sleep` is
  not a cmd builtin, so on Windows that cell died instantly, there was no run
  in flight to type during, and every assertion in
  `an_edit_during_a_run_is_not_lost` passed without the race it exists for
  ever being created.
  `crates/hickory-cli/tests/up_loop.rs` —
  `an_edit_saved_in_a_woven_file_lands_in_the_document` drives the real binary
  and saves the way an editor does (write sibling, rename over target);
  `two_regions_edited_in_one_save_both_land` covers property 2;
  `an_edit_saved_in_the_document_reaches_the_woven_file` covers the reverse
  direction. Unit tests in `up/reverse.rs` pin the diff shapes (replacement,
  pure insertion, pure deletion, two regions).
- The app's output pane is the same claim arriving over HTTP instead of the
  filesystem, and property 3 binds it too: the diff must be computed against
  the bytes the server holds, not against anything fresher or staler. A real
  bug (2026-08-17, found editing `debugging.md` in the desktop app): the pane
  reset its diff baseline to the first-load weave on every React render, so
  each save re-sent every earlier edit and the document gained a duplicate of
  them all per save — observed live as "anybody stepping at all. Truly
  nobody. And again. Truly nobody." after two inserts. Fixed by moving the
  baseline into `apps/web/src/lib/outputSave.ts`, which advances it only on
  load and on a successful POST, and serializes saves so each diff sees the
  previous save's result. `apps/web/src/lib/outputSave.test.ts` pins the
  duplicate-resend, failed-save-retry, overlapping-save, and reload cases;
  `crates/hickory-cli/tests/serve_local.rs::a_prose_edit_in_the_woven_markdown_lands_in_the_document`
  pins the server half for the weave file specifically (listed in
  `/outputs`, non-synthetic prose provenance, edit lands, re-weave
  reproduces it). A refused save is now also said out loud: the pane's
  `role="alert"` error line gained styling (`.generated-view__error`) so a
  422 cannot pass for success.
- Caveat requiring review: convergence is asserted after the writes stop, not
  during them. A pathological writer that never pauses could in principle keep
  the loop permanently behind; nothing here proves it cannot, and the honest
  contract is eventual agreement rather than bounded latency.
- **A half-written save could reach the document, found 2026-08-19 by running
  this suite on macOS for the first time.** The loop debounced on *quiet* — it
  collected watcher events until the directory had been still for 120ms — and
  quiet is not the same question as "has this file finished being written". An
  editor that truncates and dribbles its buffer in stalls longer than that, the
  batch fires on a half-written file, and the loop maps it back: the document
  lost `def describe():` entirely, and on the CI runner it stayed lost.
  `up/mod.rs::settle_batch` now waits for every path in the batch to hold still
  for a continuous window before anything reads it, which is `ingest.rs`'s
  `has_settled` idea applied to the other place the loop reads a file somebody
  else is writing.

  Two things about that fix are worth keeping in mind. **Two equal polls are not
  stability** — a paused writer looks identical either side of a short poll, and
  the first version of this fix passed its own test for that reason; the window
  has to be continuous and longer than the pauses a writer takes. And **300ms is
  a heuristic**, comfortably above a real editor's sub-millisecond gaps and below
  what a person notices on save, but a writer that stalls longer still defeats
  it — the budget bounds the wait, and the events its later writes produce bring
  the loop back to correct the document.

  **The test that caught it was passing for the wrong reason**, which is the more
  useful half. It paused 30ms between chunks against a 120ms debounce, so the
  batch could only fire mid-write on a machine slow enough to stall a chunk past
  120ms — it passed by arithmetic rather than by the product being correct, and
  its own comment claimed the pauses were "long enough for the debounce to fire".
  It also wrote 12-byte chunks of a 60-byte file and, when that was first
  "fixed" to 64-byte chunks, wrote the whole file in one go and proved nothing.
  It now pauses longer than the debounce, chunks smaller than the file, and
  watches the document *during* the write rather than after — the violation is
  transient, because the last chunk heals it, so asserting on the final state
  only catches it at random.
