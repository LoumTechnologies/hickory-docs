# An Output Volume Flushes Only What The Run Changed

Given a volume declared with an `output=` path and seeded from a directory,
when the run finishes and the volume is flushed to disk, then only the files
a cell created or changed since the volume was seeded are written; a file
whose bytes are identical to its seeded bytes is left alone.

Before this, a volume seeded from `.` flushed everything back: the document
itself, and — the case that was found — a placeholder `[never run]` staged
before the run for a file a cell was about to fill, which the flush then
wrote over the cell's real product. Every `<hick:file>` filled by a cell
that also mounted `.` with `output="."` came out as `[never run]` from a run
that had produced it, with the run reporting nothing failed. A file the run
did not touch is not that run's output.

---

Last LLM verification:
- Date: 2026-09-02
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hick-literate/src/volume_state.rs` (`VolumeStore::seeded`,
  recorded by `seed_from_directory` and `seed_tar`);
  `crates/hick-literate/src/lib.rs` `flushable_volume_files` skips entries
  equal to their seeded bytes, at both the per-cell and the post-loop flush.
- Test coverage: `crates/hick-literate/src/lib.rs::tests`
  (`a_flush_writes_only_what_the_run_changed`);
  `crates/hickory-cli/tests/round_trip.rs`
  (`a_chain_that_regenerates_a_client_from_a_spec_is_stable_across_runs_and_weaves`,
  whose cells mount `.` with `output="."`).
