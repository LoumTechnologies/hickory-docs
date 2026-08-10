# Recordings Are Written Only When A Run Is Asked To Write One, And Never By `check`

Given a document with `<hick:exec>` cells, when it is driven through the
shipped `hickory` binary, then:

- `hickory run <doc>` with no cache flag **executes every cell and records
  nothing**. It asks *"what is the answer now"*, so it neither reuses a
  recording run-wide nor leaves one behind. The project's
  `.hick-cache/transcripts/` is still opened when it already exists, because a
  cell that declares `freeze="true"` needs it to find the recording it is
  checked against — read-only, in both directions.
- `hickory run --cache <doc>` **records every cell it executes** into
  `.hick-cache/transcripts/<container>/<key>.json`, creating that directory
  when it does not exist, and answers a cell from its recording while the
  recording still matches the cell's cache key. This is the **only** command
  that writes a recording.
- `hickory run --freeze <doc>` makes freeze the **run-wide default**: every
  cell that does not declare otherwise is answered from its recording and
  never executed. A cell with no recording **fails the run** — it is never
  silently executed, and never recorded on the spot. A cell's own
  `freeze="false"` still wins.
- `hickory check <doc>` has a `--freeze` flag and **deliberately has no
  `--cache`**. A verifier that can write the baseline it then compares against
  verifies nothing; that circularity is exactly what the `unverifiable`
  outcome exists to prevent. `check` reports a cell with no baseline as
  unverifiable (exit `2`) rather than establishing one.

The whole freeze lifecycle is therefore reachable from the CLI alone: set
`freeze="false"`, run `hickory run --cache <doc>` once to record the cell,
restore `freeze="true"`, and `hickory check <doc>` verifies it. A recording is
keyed by the container image, capabilities, command text, and secret names —
not by the `freeze` attribute — so freezing a cell after recording it does not
retire its recording.

Because a frozen cell is never executed, `--cache` alone cannot record one:
recording requires letting it run. Every user-facing message about a missing
recording must say so, and must name a command the shipped binary really
accepts — `hickory run --cache`, never `hick run --cache` (a binary that has
never existed).

---

Last LLM verification:
- Date: 2026-08-10
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: the flags are declared on `RunArgs` (`--cache`, `--freeze`) and
  `CheckArgs` (`--freeze` only) in `crates/hickory-cli/src/main.rs`, and are
  carried as `hickory_cli::CachePolicy` into `run_doc_cached`
  (`crates/hickory-cli/src/lib.rs`). `run_doc` keeps its old signature and
  delegates with `CachePolicy::OFF`, so every embedded caller (the server's
  preview and run paths, watch, the agent's tools, the existing integration
  tests) is unchanged: executes everything, records nothing. `run_doc_cached`
  builds `cache::CacheConfig::new(project_dir, policy.cache || policy.freeze,
  policy.freeze)`, creates the transcript directory when `--cache` is set (a
  fresh project would otherwise record nothing on its first `--cache` run),
  and hands the config to `run_pipeline_live` whenever either flag is set even
  if the directory does not exist — which is what makes `--freeze` against an
  unrecorded project fail loudly instead of executing. `cmd_check` passes
  `cache: false` unconditionally; there is no code path from `check` to
  `cache::cache_store`. The per-cell decision itself is unchanged and lives in
  `run_pipeline_live` (`crates/hick-literate/src/lib.rs`) — see
  `freeze-is-declared-per-cell.md`.
- Test coverage: `crates/hickory-cli/tests/cache_flags.rs` drives the shipped
  binary — `run_without_the_flag_records_nothing`,
  `run_cache_writes_a_recording`,
  `a_frozen_cell_gets_its_baseline_entirely_through_the_cli`,
  `run_freeze_serves_the_recording_instead_of_executing` (proved by doctoring
  the recording on disk and finding the doctored text in the woven output),
  `run_freeze_without_a_recording_fails_and_names_the_recording_command`,
  `check_refuses_a_cache_flag`, and
  `check_freeze_reports_an_unrecorded_cell_as_unverifiable`. The message
  wording is pinned by `check_exits_unverifiable_when_a_cell_has_no_baseline`
  in `crates/hickory-cli/tests/check_tests.rs` (must contain `hickory run
  --cache`, must not contain `hick run --cache`) and by
  `a_frozen_cell_without_a_recording_fails_with_an_actionable_error` in
  `crates/hick-literate/tests/freeze_tests.rs`.
- Caveat requiring LLM review: nothing mechanically stops a future `--cache`
  flag from being added to `check`; `check_refuses_a_cache_flag` catches it
  only because clap rejects unknown arguments. Also out of scope by decision:
  hickory issue #10 proposes that a frozen cell with no recording should
  record on first run instead of erroring. Until that is decided, "a frozen
  cell is never recorded by `--cache`" is deliberate, and the docs
  (`docs/hick-guide.hick` §13, `README.md` Reference) must keep explaining the
  un-freeze/record/re-freeze sequence.
