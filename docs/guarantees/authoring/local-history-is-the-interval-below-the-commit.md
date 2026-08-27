# Local History Is The Interval Below The Commit

Given a folder of documents, when anything writes to it — you, a run, a weave,
a reverse edit, an ingest, the agent, find-and-replace, the merge driver —
then what the affected files held immediately before is kept, locally, and can
be put back:

- **The unit is the act, not the file.** Almost every writer here is a batch
  writer: a replace touches forty files, an ingest lands a scaffolder's tree, a
  weave rewrites every `.md`. Per-file entries would record that correctly and
  make it useless — undoing a replace would be forty reverts in the right
  order, by hand, from memory. A file's own timeline is a **filter** over the
  acts that touched it, never a second structure.
- **Every act says who made it**, because "is going back here sane?" has a
  different answer per kind.
- **A generated write is shown and compared, never reverted.** The next run
  would undo the revert, so the verb is refused with the reason rather than
  offered and broken.
- **A file that has moved on is reported, never silently skipped.** Somebody
  undoing a batch is already unsure what happened, and a quiet partial revert
  is how they come to trust a state that never existed.
- **Going back is itself an act**, recorded like any other. A history you can
  fall out of is a history nobody trusts.
- **It is bounded and forgettable.** Evicted oldest-first under a byte budget
  and an age limit (`HICKORY_HISTORY_BYTES`, `HICKORY_HISTORY_DAYS`), and
  `hick history forget` exists because this store holds bytes their author
  never chose to keep.

**It is a cache, not a record, and nothing may cite it.** No `from=`, no
`cites=`, no attribute in the hick grammar takes a local-history address. It
never crosses git and never crosses the peer channel; two of your machines
have two different local histories and that is correct.

Say **"local history"**. Never "version history", "backup", or "snapshots" —
each promises durability across machines, which this deliberately does not
have, and the promise is the harm.

---

Last LLM verification:
- Date: 2026-08-27
- Reviewer: Claude (Opus 5)
- Result: verified by running it, with one real bug found that way
- Evidence:
  - The store is `crates/hickory-workspace/src/history.rs`: content-addressed
    blobs, an **append-only** act log (an interrupted write loses the tail and
    never the middle, which matters because this is the store somebody reaches
    for after a crash), reference-counted sweeping, and `forget`.
  - `crates/hickory-cli/src/history.rs` is the seam every writer goes through.
    It swallows its own failures on purpose: a run must not fail because a
    cache on this machine could not be written.
  - Recorded at: **both** output writers — `write_outputs_detailed` and the
    `hick up` loop's `WovenState::write_output`, because a rule applied to only
    one of them is a rule with a hole in it — plus find-and-replace
    (`serve/find.rs`), the reverse edit (`up/reverse.rs` and
    `write_source_by_doc_path`), `hick ingest` (`ingest_exec.rs`), the merge
    driver, saves and anchored-terminal typing (`write_source_as`).
  - CLI: `hick history`, `hick history <path>`, `show`, `revert`, `forget`.
- Test coverage:
  - `crates/hickory-workspace/src/history.rs` — 13 unit tests: an act holds
    every file it touched while a file is a filter over acts, an act that
    changed nothing is not recorded, a replace reverts whole, one file out of
    a batch reverts alone, a created file is removed again, a generated write
    is refused with a reason, identical bytes are stored once, an ambiguous id
    prefix resolves to nothing, a torn last line loses only itself, old acts
    are evicted and their blobs swept while live blobs are not, forgetting one
    path leaves the rest of a batch **and the bytes are actually gone**, and
    date arithmetic crosses a month, a year and a leap day.
  - `crates/hickory-cli/tests/serve_find.rs` — through the real server: a
    replace is **one** act holding both files and undoes whole, and a file
    somebody edited afterwards is reported rather than overwritten. This is
    the sentence at the end of
    `docs/guarantees/search/find-and-replace-is-exhaustive.md`, closed.
- Verified by running: `hick run`, `hick history`, `hick history show`, and
  `hick history revert` against a scratch project.
  **The bug that found:** `hick run note.hick` gives a document parent of `""`,
  which hashed to the project key for the empty string — so every act landed
  in a store nothing would ever look in, while `hick history` read the store
  for the working directory and reported an empty folder. Two stores, no
  error, and a feature that silently did nothing. Roots are absolutised at the
  seam now, with a test.
- Caveat requiring review:
  - **There is no app panel.** The spec describes one in the same family as
    the git pane, with the act list on the left and a diff on the right. The
    mechanism and the CLI are built; the surface is not, so today this is a
    command-line feature.
  - **`typed` acts are not coalesced, because nothing records them yet.** The
    spec's hardest open question — how long a quiet gap cuts a burst of typing
    into two acts — is untouched: the only `Typed` acts are the anchored
    terminal's, which are already one line each. A buffer-level recorder would
    have to answer it.
  - **`external` and `refactor` are defined and unrecorded.** The kinds exist
    and nothing writes them: the file watcher sees external edits
    (`the-app-sees-external-edits.md`) and does not record them, and refactor
    mode does not either. Those two rows of the spec's table are still
    "nothing" as a way back.
  - **Retention defaults are unmeasured, exactly as the spec warns.** 256 MB
    and 14 days were chosen to be obviously bounded rather than obviously
    right, and nothing here was watched on a real folder for a week.
  - **Eviction is O(acts) in the worst case.** Pruning by size drops one act
    at a time and re-measures, because a blob is only reclaimed when the last
    act naming it goes. On a store that is far over budget that is a lot of
    rewriting, and it has only been run against tiny fixtures.
  - **Reverting while a run or an output edit is in flight is not ordered.**
    The spec says the safe answer is to refuse rather than interleave. Nothing
    refuses; the two have simply never been done at once.
