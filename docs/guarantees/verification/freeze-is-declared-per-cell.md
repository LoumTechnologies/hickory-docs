# Freeze Is Declared Per Cell, With The Run-Wide Flag As Its Default

Given a document whose `<hick:exec>` and `<hick:agent>` cells may carry a
`freeze` attribute, when the pipeline runs, then each cell's freeze state is
decided independently:

- `freeze="true"` — the cell **executes at most once, ever**. It is answered
  from its recording whenever one matches its cache key. When none does,
  `hickory run` executes the cell that one time and records it, so a cell can
  be declared frozen from the moment it is written — no flag, and no edit to
  the document, is required to establish the baseline. `hickory test` never
  records: it reports such a cell as **unverifiable** (exit `2`) instead.
- `freeze="false"` — the cell is **always executed** and is never satisfied
  from a recording, *even when the run was started with `hickory run
  --freeze`*.
- attribute absent — the cell inherits the run-wide default: frozen under
  `hickory run --freeze`, recorded-and-reused under `hickory run --cache`,
  live otherwise.

This is **one setting on one axis**, not two booleans: what a missing
recording means. `cache::CacheMode` names the three answers — `Off` (nothing;
execute and remember nothing), `Reuse` (remember this), `Require` (no baseline
yet) — and there is no impossible fourth state to defend against. The run-wide
flags and the per-cell attribute select values of the same enum, which is why
they compose without special cases.

Where a recording is written, and by whom, is a separate guarantee:
`recordings-are-written-only-when-asked-for.md`.

A value that is neither `true` nor `false` is rejected at DAG-build time rather
than being treated as false, because a typo in a verification switch must never
quietly disable verification.

Freezing one cell must not freeze the run, and a run-wide freeze must remain
overridable per cell in both directions.

A run with no recording directory at all — the server's live preview, watch
mode, the agent's own tool calls — can neither replay a frozen cell nor record
one. There the declaration cannot be honoured: the cell executes and a warning
says so. Failing a live preview over a cell that would run fine under `hickory
run` helps nobody.

## The honest cost of freeze

A frozen cell's `<hick:expect>` assertions are evaluated against the **cached**
output on every run after the first, so they pass trivially — nothing was
executed and nothing in the world was consulted. Freeze verifies *"the document
still produces what we recorded"*, not *"the world still agrees"*. A suite in
which every cell is frozen is not a passing suite; it is a suite that did not
run. This is the trade a cell makes deliberately when its output legitimately
moves over time (lockfiles, network fetches, timestamps); an integration test
must never make it.

`hickory test` is the caller that refuses to establish a baseline: it reports a
frozen cell with no recording as **unverifiable** (exit `2`) so every such cell
is listed in one pass, and writes nothing. A frozen cell that DOES have a
recording is verified, not unverifiable — the recording is its baseline. See
`test-separates-unverifiable-from-drifted.md`.

---

Last LLM verification:
- Date: 2026-08-10
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: the attribute is parsed by `parse_freeze` in
  `crates/hick-exec/src/dag.rs` into `ExecInfo.freeze: Option<bool>`, rejecting
  non-boolean values with `DagValidationError::InvalidFreeze`. The three-valued
  setting is `cache::CacheMode` (`crates/hick-literate/src/cache.rs`);
  `CacheConfig::mode` is the run-wide default and `CacheConfig::mode_for`
  applies the cell's own attribute over it (`Some(true)` → `Require`,
  `Some(false)` → `Off`). `cache::cell_mode` degrades to `Off` when the run has
  no `CacheConfig` at all. `run_pipeline_live`
  (`crates/hick-literate/src/lib.rs`) consults a recording when
  `cell_mode.consults()`, and on a miss either stops (only when
  `PipelineConfig::collect_unverifiable`, i.e. `hickory test`) or falls through
  to execution; the post-execution `cache_store` fires when
  `cc.mode.records() || cell_mode.records()` and never when
  `collect_unverifiable` is set. `run_doc_cached`
  (`crates/hickory-cli/src/lib.rs`) always hands the pipeline a `CacheConfig`
  rooted at the document's project directory — including for a flagless
  `hickory run`, which is what lets a `freeze="true"` cell write its first
  recording — and `hick_literate::cache_mode` maps the `--cache` / `--freeze`
  flags onto the enum (`--freeze` wins when both are given).
- Test coverage: `crates/hick-literate/tests/freeze_tests.rs` —
  `a_frozen_cell_does_not_freeze_the_run`,
  `a_frozen_cell_serves_its_recording_without_executing`,
  `a_frozen_cell_without_a_recording_runs_once_and_records_itself`,
  `editing_a_frozen_cell_re_records_it_on_the_next_run`,
  `a_frozen_cell_that_ran_once_is_not_re_recorded_over`,
  `a_run_wide_freeze_serves_what_is_recorded_and_records_what_is_not`,
  `a_cell_can_opt_out_of_a_run_wide_freeze`,
  `an_opted_out_cell_re_executes_instead_of_reusing_its_recording`,
  `a_non_boolean_freeze_value_is_rejected`, and
  `a_frozen_cell_without_any_cache_directory_executes_rather_than_failing`.
  Through the shipped binary:
  `a_frozen_cell_gets_its_baseline_entirely_through_the_cli` and
  `a_frozen_cell_is_the_only_thing_a_flagless_run_records`
  (`crates/hickory-cli/tests/cache_flags.rs`), and
  `run_records_a_cell_frozen_from_the_start_and_test_then_verifies_it`
  (`crates/hickory-cli/tests/test_command_tests.rs`). Attribute parsing is
  covered by `freeze_attribute_parses_both_values`,
  `freeze_attribute_defaults_to_inherit`, and
  `freeze_attribute_rejects_non_boolean` in `crates/hick-exec/src/dag.rs`.
- Caveat requiring LLM review: the "honest cost" above is a documentation
  guarantee, not a mechanical one — nothing in the code stops an author from
  freezing every cell in a document and reading the resulting green run as
  verification. The docs (`docs/hick-guide.hick` §13,
  `docs/docs/hick-guide.md`) must keep saying so. Nothing mechanical stops the
  warning for a frozen cell in a cacheless run from going unnoticed either: an
  embedded caller that shows no logs shows no warning.
