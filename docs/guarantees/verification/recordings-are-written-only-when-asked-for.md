# Recordings Are Written Only When A Run Is Asked To Write One, And Never By `test`

Given a document with `<hick:exec>` cells, when it is driven through the
shipped `hickory` binary, then:

- `hickory run <doc>` with no cache flag **executes every cell and records
  nothing — except a cell that asked to be recorded.** It asks *"what is the
  answer now"*, so no ordinary cell is answered from a recording or leaves one
  behind. A cell that declares `freeze="true"` is the one exception, and it
  asked for the exception itself: it has no baseline until a run writes one, so
  this run executes it exactly once and records it under
  `.hick-cache/transcripts/`, creating that directory if needed. Every later
  run replays it.
- `hickory run --cache <doc>` **records every cell it executes** into
  `.hick-cache/transcripts/<container>/<key>.json`, and answers a cell from its
  recording while the recording still matches the cell's cache key. It extends
  to the whole document what `freeze="true"` asks for one cell.
- `hickory run --freeze <doc>` makes freeze the **run-wide default**: every
  cell that does not declare otherwise is answered from its recording rather
  than executed. A cell with no recording has no baseline yet, so it is
  executed once and recorded, exactly as a cell-declared freeze is. A cell's
  own `freeze="false"` still wins.
- `hickory test <doc>` **writes no recording under any flag**, and
  deliberately has no `--cache`. A verifier that can write the baseline it then
  compares against verifies nothing; that circularity is exactly what the
  `unverifiable` outcome exists to prevent. `test` reports a cell with no
  baseline as unverifiable (exit `2`) rather than establishing one. This is
  therefore the command that answers *"is everything already recorded?"* —
  `hickory test --freeze` asserts it for every cell in the document.

The whole freeze lifecycle is therefore reachable from the CLI alone, in one
step: write the cell with `freeze="true"`, run `hickory run <doc>` once, and
`hickory test <doc>` verifies it. There is no un-freeze / record / re-freeze
sequence any more, and **no user-facing message may describe one**. A recording
is keyed by the container image, capabilities, command text, and secret names —
not by the `freeze` attribute — so editing the command retires the recording
and the next `hickory run` records the new one.

Every user-facing message about a missing recording must name a command the
shipped binary really accepts — `hickory run`, never `hick run --cache` (a
binary that has never existed).

---

Last LLM verification:
- Date: 2026-08-10
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: the flags are declared on `RunArgs` (`--cache`, `--freeze`) and
  `TestArgs` (`--freeze` only) in `crates/hickory-cli/src/main.rs`, and are
  mapped by `hick_literate::cache_mode` onto the single three-valued
  `cache::CacheMode` (`Off` / `Reuse` / `Require`) carried into
  `run_doc_cached` (`crates/hickory-cli/src/lib.rs`). `run_doc` keeps its old
  signature and delegates with `CacheMode::Off`, so every embedded caller (the
  server's preview and run paths, watch, the agent's tools) still executes
  everything and records nothing. `run_doc_cached` always builds
  `cache::CacheConfig::new(project_dir, cache_mode)` and always hands it to
  `run_pipeline_live`: gating on the directory already existing would make the
  first run of a frozen cell in a fresh project silently record nothing.
  `cmd_test` passes `Require` only for `--freeze` and never `Reuse`, and
  `RunMode::Verify` sets `PipelineConfig::collect_unverifiable`, which in
  `run_pipeline_live` (`crates/hick-literate/src/lib.rs`) both turns a
  `Require` miss into a `never_run` entry and suppresses every `cache_store`
  call — so `test` has no path to a recording even under `--freeze`, where the
  run-wide mode would otherwise permit one for an opted-out cell.
- Test coverage: `crates/hickory-cli/tests/cache_flags.rs` drives the shipped
  binary — `run_without_the_flag_records_nothing`,
  `run_cache_writes_a_recording`,
  `a_frozen_cell_gets_its_baseline_entirely_through_the_cli` (frozen from the
  start, one plain `hickory run`, then exit 0),
  `a_frozen_cell_is_the_only_thing_a_flagless_run_records`,
  `run_freeze_serves_the_recording_instead_of_executing` (proved by doctoring
  the recording on disk and finding the doctored text in the woven output),
  `run_freeze_records_a_cell_that_has_no_recording_yet` (records, then replays
  the doctored recording), `test_refuses_a_cache_flag`, and
  `test_freeze_reports_an_unrecorded_cell_as_unverifiable`. The message wording
  is pinned by `test_exits_unverifiable_when_a_cell_has_no_baseline` in
  `crates/hickory-cli/tests/test_command_tests.rs` (must name `hickory run`,
  must not contain `hick run --cache`, and must not still describe restoring a
  freeze attribute), and the end-to-end lifecycle by
  `run_records_a_cell_frozen_from_the_start_and_test_then_verifies_it` in the
  same file, which also asserts that the `hickory test` run preceding it left
  no `.hick-cache/` behind.
- Caveat requiring LLM review: nothing mechanically stops a future `--cache`
  flag from being added to `test`; `test_refuses_a_cache_flag` catches it only
  because clap rejects unknown arguments. The suppression of `cache_store`
  under `collect_unverifiable` is the real guard, and it is keyed on a
  `PipelineConfig` field a future caller could set differently.
