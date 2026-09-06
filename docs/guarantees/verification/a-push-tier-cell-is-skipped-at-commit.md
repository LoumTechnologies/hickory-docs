# A Cell Tiered `push` Is Skipped At The Commit Tier, And Costs No Exit Code

Given a document whose `<hick:exec>` (or `<hick:agent>`) cells may carry a
`tier` attribute, when a run is started with `--tier commit`, then each cell's
tier decides whether it may execute:

- `tier="commit"`, **or the attribute absent** — the cell runs at every gate,
  exactly as it did before tiers existed. This is the default because the safe
  value of a switch that decides how much gets checked is the one that checks
  more.
- `tier="push"` — the cell is **not executed**. It is still looked up in
  `.hick-cache/transcripts/`:
  - **A recording whose key still matches answers it**, precisely as a frozen
    cell's recording does. Everything downstream is then verified normally:
    its `<hick:expect>` assertions are evaluated against the recorded output,
    and the woven markdown is compared for drift.
  - **With no matching recording the cell is skipped.** Its expectations are
    not evaluated. The outputs that embed its transcript — the document's
    woven markdown, and any `hick:file` product provenance says it fed — are
    neither compared for drift nor written over. **Every other output of the
    document is still compared**: the tangle is what a commit is for.
- Any other value is an error at DAG-build time naming both accepted values,
  for the same reason `freeze="yes"` is: a tier nobody recognises would
  otherwise silently become the tier that runs least.

Without `--tier`, and with `--tier push`, nothing changes: every cell runs.
`--tier` naming neither value is refused before anything runs, naming both.

**A skip is not an outcome.** `hick test --tier commit` exits `0` when nothing
else failed. It is deliberately *not* exit `2` (unverifiable): unverifiable
means a baseline nobody ever established, which is a problem somebody must
fix, and a skip is a decision the document made on purpose, which is not. A
run that skips cells reports each one — document, line, container, the cell's
first command line — and closes with a count and the command that does run
them (`hick test .`). Precedence is unaffected: if the same run also drifts,
it exits `1`.

**A skip never destroys an artifact.** `hick run --tier commit` preserves the
outputs a skipped cell feeds rather than writing `[never run]` over bytes a
real run produced — the same rule, and the same reasoning, as
`a-weave-without-a-recording-keeps-the-artifact.md`.

Rationale: a literate repository of 760 documents, 199 of which run
`cargo test` for 5–30 seconds, takes 52 minutes to verify in full and under a
minute to verify its tangles and weaves. A pre-commit hook costing an hour is
a hook that gets uninstalled, taking the drift gate with it. Tiers let the
*document* say which of its cells a commit is worth waiting for, so the gate
that runs on every commit is the one people keep.

---

Last LLM verification:
- Date: 2026-09-06
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence:
  - The attribute is `hick_exec::attrs::CellTier` (`crates/hick-exec/src/attrs.rs`),
    parsed by `parse_tier` into `dag::ExecInfo::tier` at all three extraction
    sites (`extract_exec_info`, `extract_script_info`, `extract_agent_info` in
    `crates/hick-exec/src/dag.rs`), rejecting anything else with
    `DagValidationError::InvalidTier`, whose message names `tier="commit"` and
    `tier="push"`. `CellTier` derives `Ord` as `Commit < Push`, and
    `CellTier::runs_at(gate)` is `self <= gate` — the whole tier rule in one
    comparison.
  - The gate is `hick_literate::tier::TierGate` (`crates/hick-literate/src/tier.rs`),
    `Push` by default so every existing caller is unchanged, parsed from the
    flag by `TierGate::parse` (its error names both values) and carried on
    `PipelineConfig::tier`.
  - In `run_pipeline_live` (`crates/hick-literate/src/lib.rs`) both cell
    branches — the agent branch and the exec branch — compute
    `tier_skipped = config.tier.skips(exec_info.tier)` and force `cell_mode`
    to `cache::CacheMode::Require` when it is set, which is what makes a
    skipped cell consult its recording. On a hit the existing cache-hit path
    runs unchanged (expectations evaluated, transcript injected). On a miss a
    `tier_skipped` arm — placed *before* the `collect_unverifiable` arm, so a
    tiered-out frozen cell is skipped rather than reported unverifiable —
    records the cell in `PipelineResult::skipped` and continues. A third check
    after the lookup block covers a run with no cache config at all.
  - `PipelineResult::skipped` is a `SkippedCells` (`CellId` → first command
    line). `PipelineResult::outputs_from_skipped_cells` and
    `outputs_missing_a_recording` share one provenance scan (`outputs_fed_by`)
    in `crates/hick-literate/src/tier.rs`.
  - `hickory_cli::tier::outputs_not_compared` (`crates/hickory-cli/src/tier.rs`)
    is the drift and write exemption: the provenance-fed outputs plus the
    document's weave target, which always embeds every transcript and is
    therefore named directly rather than inferred. `check_failures` skips
    those paths (so no `Drift` failure is manufactured) and
    `write_outputs_detailed` preserves them (so no artifact is overwritten),
    both in `crates/hickory-cli/src/lib.rs`.
  - `hickory_cli::tier::report_skips` prints one `SKIP` line per cell and
    `skip_summary` the closing count; `cmd_test` (`crates/hickory-cli/src/main.rs`)
    calls both and leaves `CheckOutcome` untouched, which is what keeps the
    exit code `0`.
  - The hook: `HOOK_BODY` in `crates/hickory-cli/src/init.rs` runs
    `hick test --tier commit $hick_docs` — one process — and prints the
    skipped count with `hick test .` as the gate that runs them.
- Test coverage:
  - Attribute parsing: `tier_attribute_parses_both_values`,
    `tier_attribute_defaults_to_commit`,
    `tier_attribute_rejects_any_other_value`, and
    `a_commit_cell_runs_at_every_gate_and_a_push_cell_does_not` in
    `crates/hick-exec/src/attrs.rs`.
  - The gate: `the_push_gate_runs_everything`,
    `the_commit_gate_skips_only_push_cells`,
    `an_unknown_gate_is_refused_naming_both` in
    `crates/hick-literate/src/tier.rs`.
  - End to end through the shipped binary,
    `crates/hickory-cli/tests/cell_tiers.rs`, over one document whose
    `tier="push"` cell claims something untrue of its own output — so any gate
    that ran it would exit `3`:
    `commit_tier_skips_a_push_cell_and_still_exits_zero` (exit `0`, the skip
    named, no `FAIL`), `the_same_document_fails_at_the_push_tier` (exit `3`
    with no `--tier` and with `--tier push`, and nothing skipped),
    `a_recording_answers_a_push_cell_at_the_commit_tier` (after
    `hick run --cache`, the commit tier verifies from the recording and
    catches the false claim), `an_output_no_skipped_cell_feeds_is_still_checked_for_drift`
    (exit `1` for a hand-edited tangled product while a cell is skipped), and
    `an_unknown_tier_is_refused_naming_both`.
- Caveats requiring LLM review:
  - **The artifact-preservation half is reviewed, not tested.**
    `write_outputs_detailed`'s new `untouched` check mirrors the
    `missing_recording` one exactly and is covered by neither
    `weave_keeps_artifacts.rs` nor `cell_tiers.rs`; a test would need
    `hick run --tier commit` over a document whose push cell feeds a
    committed product.
  - **The agent branch is reviewed, not tested.** `<hick:agent tier="push">`
    takes the same two code paths, but the CLI test suite has no agent-cell
    tier test — an agent cell needs a runner or a recording to be interesting.
  - **The hook's skip line is reviewed, not tested.** `init_tests.rs` covers
    the exit-code arms; the `grep -c '^SKIP '` count depends on the exact
    prefix `report_skips` prints, and nothing mechanical ties the two
    together. Changing that prefix silently breaks the hook's count (not its
    gating).
  - **A cell that DEPENDS on a skipped cell is not handled specially.** The
    skipped cell still contributes its cache key to its successors (the key is
    computed and recorded before the lookup, so downstream keys do not move),
    but a `tier="commit"` cell reading a volume the skipped cell would have
    written runs against whatever is there. Tiering a cell others depend on is
    an authoring mistake this does not catch; the natural place for it is
    DAG-build time, where the graph is already known.
  - Tiers do not compose with `--freeze` in any special way: a `freeze="true"
    tier="push"` cell with no recording is *skipped* at the commit tier rather
    than reported unverifiable. That is a judgement call — the commit gate was
    told not to run this cell, so it has no business complaining that nobody
    ran it — and `hick test` without `--tier` still reports it as
    unverifiable, which is where that complaint belongs.
