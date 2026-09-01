# An Earlier Cell's Output Survives A Later Cell's Failure

Given a `hick run` where a cell mounting an output volume genuinely
succeeds, and a LATER cell in the same run then fails, then that earlier
cell's real output is inspectable on disk under `.hick-cache/last-run/` —
never silently gone just because the run, as a whole, did not finish.

## Why

Confirmed directly in `crates/hick-literate/src/lib.rs`: output-volume files
were flushed into `PipelineResult::files` exactly once, in a block AFTER the
entire exec loop finished. `exec_result?` inside that loop propagates any
cell's failure immediately out of the whole function, so a cell failing
after several earlier cells had already succeeded — with their volumes
already extracted into the in-memory `volume_store` — still left the caller
with nothing on host disk for those earlier cells, `hick:file` content
aside (which is written by a separate, earlier, unconditional pass and is
unaffected by this at all).

This mattered in practice: an author iterating on a multi-cell document —
build the scaffold, then extend it, then run a self-test — could not inspect
what the scaffold step had actually produced while the self-test step was
still broken, because nothing reached disk until every cell passed. Found
writing a tutorial as a `.hick` document, where this was the single largest
source of "why can't I see what my own cell just did" friction.

## What the fix is, and what it deliberately is not

`PipelineConfig::on_volume_flush` (`crates/hick-literate/src/lib.rs`) is a
new, optional hook — shaped like the existing `on_exec` live-event hook —
fired once per output volume a cell's mounts touch, immediately after that
cell's own extraction, with that volume's CURRENT flushable files (computed
by `flushable_volume_files`, now also the implementation the post-loop
consolidated pass uses, so there is exactly one place that turns a volume's
tar bytes into prefixed `FileContent`). Never fired for a volume the
document has already ingested, matching the consolidated pass's own
exclusion — the document owns those bytes.

`hickory-cli`'s `volume_flush_mirror` (`crates/hickory-cli/src/lib.rs`)
registers this hook to write each file under
`.hick-cache/last-run/<the same relative path a full write would use>` —
**never the real output tree**. That was a deliberate, considered choice,
not the first design tried:

- `write_outputs_detailed` — the function that DOES write the real tree —
  owns real correctness properties an incremental, best-effort hook has no
  business reimplementing in parallel: `record_generated_writes` (the
  local-history "before" snapshot, which must run before ANY write reaches
  the real tree, or its own before/after diff is corrupted by a write that
  got there first), the missing-recording preservation check (never
  overwriting existing content when a cell's baseline cannot be found), and
  read-only clearing.
- Writing straight to the real tree from the incremental hook would mean
  either duplicating all of that (a second implementation of the same
  safety properties, guaranteed to drift from the first) or skipping it (a
  real regression to local history and to the missing-recording guarantee).
- `.hick-cache/` is already gitignored by `hick init` and already this
  tool's own bookkeeping location (transcripts live there today) — using it
  for a scratch mirror needed no new convention.

A fully successful run is unaffected either way: `write_outputs_detailed`
still runs exactly as before and still owns the real tree. The mirror is
strictly additive visibility, not a second source of truth.

---

Last LLM verification:
- Date: 2026-09-01
- Reviewer: Claude (Sonnet 5)
- Result: verified
- Evidence: `crates/hick-literate/src/lib.rs`'s `VolumeFlushHook`,
  `PipelineConfig::on_volume_flush`, the hook-firing call site inside the
  per-cell mount-extraction loop, and `flushable_volume_files` (shared with
  the post-loop pass). `crates/hickory-cli/src/lib.rs`'s
  `volume_flush_mirror`, wired into the one `PipelineConfig` construction
  `hick run`/`hick test` go through.
- Test coverage:
  `crates/hickory-cli/tests/volume_flush_survives_failure.rs` (2 tests,
  driving the real binary): a document with a succeeding volume-writing cell
  followed by a failing cell confirms the real output tree gains nothing
  (the run failed) while `.hick-cache/last-run/` gains the first cell's real
  file, byte for byte; a second test confirms a FULLY successful run still
  writes the real tree the normal way, so the mirror is never mistaken for
  it. Confirmed load-bearing by temporarily disabling the hook and observing
  the first test fail.
- Caveat requiring LLM review: the mirror can go stale — a file a cell wrote
  in an earlier run but no longer writes in the current one stays under
  `.hick-cache/last-run/` until something overwrites or removes it. This is
  a debug convenience, not a record, so staleness is a nuisance rather than
  a correctness hazard, but nothing currently clears the directory between
  runs.
