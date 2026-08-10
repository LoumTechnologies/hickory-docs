# Freeze Is Declared Per Cell, With The Run-Wide Flag As Its Default

Given a document whose `<hick:exec>` cells may carry a `freeze` attribute, when
the pipeline runs, then each cell's freeze state is decided independently:

- `freeze="true"` — the cell is **checked against its recording and never
  executed**. If no recording matches its cache key, the run fails with an
  error naming the container, the source line, the command, and how to record
  it.
- `freeze="false"` — the cell is **always executed** and is never satisfied
  from a recording, *even when the run was started with `hick run --freeze`*.
- attribute absent — the cell inherits the run-wide default: frozen under
  `hick run --freeze`, live otherwise.

A value that is neither `true` nor `false` is rejected at DAG-build time rather
than being treated as false, because a typo in a verification switch must never
quietly disable verification.

Freezing one cell must not freeze the run, and a run-wide freeze must remain
overridable per cell in both directions.

## The honest cost of freeze

A frozen cell's `<hick:expect>` assertions are evaluated against the **cached**
output, so they pass trivially — nothing was executed and nothing in the world
was consulted. Freeze verifies *"the document still produces what we recorded"*,
not *"the world still agrees"*. A suite in which every cell is frozen is not a
passing suite; it is a suite that did not run. This is the trade a cell makes
deliberately when its output legitimately moves over time (lockfiles, network
fetches, timestamps); an integration test must never make it.

`hickory check` is the one caller that does not abort: it reports a frozen
cell with no recording as **unverifiable** (exit `2`) so every such cell is
listed in one pass. A frozen cell that DOES have a recording is verified, not
unverifiable — the recording is its baseline. See
`check-separates-unverifiable-from-drifted.md`.

---

Last LLM verification:
- Date: 2026-08-09
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: the attribute is parsed by `parse_freeze` in
  `crates/hick-exec/src/dag.rs` into `ExecInfo.freeze: Option<bool>`, rejecting
  non-boolean values with `DagValidationError::InvalidFreeze`. The decision is
  made per exec in `run_pipeline_live`
  (`crates/hick-literate/src/lib.rs`), which derives
  `(serve_from_cache, require_cache)` from `exec_info.freeze` falling back to
  `CacheConfig::{enabled, freeze}` — so `CacheConfig::freeze` is now only the
  run-wide *default*. `run_pipeline_cmd` always constructs a `CacheConfig`
  (with `enabled = cache || freeze`) so a cell-level freeze can find the
  `.hick-cache/` directory even when neither flag was passed, and `run_doc` in
  `crates/hickory-cli/src/lib.rs` hands `hickory run` the same directory
  read-only (`enabled = false`) when it exists — nothing is reused or recorded
  there, but a frozen cell can find what it is checked against. Embedded run
  paths that pass `None` reject a frozen cell with an explicit error rather
  than silently ignoring the declaration.
- Test coverage: `crates/hick-literate/tests/freeze_tests.rs` —
  `a_frozen_cell_does_not_freeze_the_run`,
  `a_frozen_cell_serves_its_recording_without_executing`,
  `a_frozen_cell_without_a_recording_fails_with_an_actionable_error`,
  `global_freeze_requires_every_cell_to_be_recorded`,
  `a_cell_can_opt_out_of_a_run_wide_freeze`,
  `an_opted_out_cell_re_executes_instead_of_reusing_its_recording`,
  `a_non_boolean_freeze_value_is_rejected`, and
  `a_frozen_cell_without_any_cache_directory_says_so`. Attribute parsing is
  covered by `freeze_attribute_parses_both_values`,
  `freeze_attribute_defaults_to_inherit`, and
  `freeze_attribute_rejects_non_boolean` in `crates/hick-exec/src/dag.rs`.
- Caveat requiring LLM review: the "honest cost" above is a documentation
  guarantee, not a mechanical one — nothing in the code stops an author from
  freezing every cell in a document and reading the resulting green run as
  verification. The docs (`docs/hick-guide.hick` §13,
  `docs/docs/hick-guide.md`) must keep saying so.
