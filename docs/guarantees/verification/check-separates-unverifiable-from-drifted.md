# `hickory check` Gives Each Kind Of Failure Its Own Exit Code

Given a document being verified with `hickory check`, when re-derivation is
compared against what is committed, then the command reports exactly one of
four outcomes and exits with that outcome's own code:

- **verified** (exit `0`) — re-derivation matches what is committed.
- **drifted** (exit `1`) — a committed output no longer reproduces, or a
  `<hick:transform>` passage is stale: someone forgot to regenerate. The fix
  is `hickory run` (or `hickory refresh`) plus a commit.
- **unverifiable** (exit `2`) — at least one cell has **no baseline at all**:
  it neither executed nor was answered from a recording, so there was nothing
  for re-derivation to be compared against.
- **expectation failed** (exit `3`) — a `<hick:expect>` did not hold: the
  document claims something untrue of what its own cells produced.

And:

- A cell served from a **recording** (`freeze="true"`, or a run-wide freeze)
  is **verified, not unverifiable** — a recording is a baseline. Only a cell
  with no recording *and* no execution is unverifiable.
- **Precedence when more than one is present**, weakest to strongest:
  `verified < drifted < unverifiable < expectation failed`. Every finding is
  still printed; only the exit code is a single verdict.
  - **Unverifiable outranks drifted**, and drift comparison for that document
    is skipped: a cell that could not run contributes nothing to the woven
    output, so the drift it would produce is a consequence of the missing
    baseline rather than a finding.
  - **A failed expectation outranks both.** It is the only outcome that
    asserts something is *definitely* wrong rather than out of date or
    unknown, and the only one no automation may act on by itself. It is also
    never contaminated by a missing baseline — an unverifiable cell never
    evaluates an expectation, so every expectation that failed belongs to a
    cell that really ran.
- **A stale `<hick:transform>` is drift, not a failed expectation.** Like a
  woven file that no longer reproduces, it says the committed bytes are out of
  date with their inputs; no claim was falsified, and the fix is to
  regenerate.
- The exit-code **numbers** are frozen where the three-outcome version left
  them, and `3` is appended rather than slotted in: renumbering `unverifiable`
  would silently change the meaning of every existing `if [ $? -eq 2 ]`. So
  the numeric order is deliberately *not* the precedence order, and the two
  are kept apart in code (`CheckOutcome`'s `Ord` is precedence;
  `CheckOutcome::exit_code` is the contract).
- The unverifiable report names **which cell** (document, source line, and
  container when it has one), **why it has no baseline**, and **what to do**,
  per `.instructions/user-facing-errors.md`. The remedy must name only flags
  the shipped binary actually accepts. Each summary line likewise names the
  outcome, its code, and the next step.
- A cell is identified by a key that does **not** assume a container, so a
  future agent cell (`docs/specs/freeform/agent-cells.md`), which has none, can
  be reported the same way.

Rationale: drift means someone forgot to regenerate; unverifiable means
nothing was ever established; a failed expectation means a claim is false. CI
must be able to respond differently — most concretely, a job may reasonably
auto-regenerate drift and must never auto-fix a false claim, which is
impossible while the two share exit `1`. Conflating any of them is how "we
have verification" quietly becomes "we have verification for the parts that
ran, and we paper over the parts that lied."

---

Last LLM verification:
- Date: 2026-08-10
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence:
  - `CheckOutcome` and `CheckOutcome::exit_code` in
    `crates/hickory-cli/src/lib.rs` define the four outcomes and their codes
    (`0` / `1` / `2` / `3`); `check_outcome` takes the maximum, and
    `CheckOutcome` derives `Ord` in the order
    `Verified < Drifted < Unverifiable < ExpectationFailed`, which is what
    makes unverifiable outrank drift and a failed expectation outrank both.
    `CheckFailure::outcome` maps `Expectation` → `ExpectationFailed`,
    `Unverifiable` → `Unverifiable`, and both `Drift` and `StaleTransform` →
    `Drifted`. `cmd_check` in `crates/hickory-cli/src/main.rs` returns
    `ExitCode::from(worst.exit_code())` and prints a distinct summary line per
    outcome naming the code and the next step. The table is repeated in
    `hickory check --help` (a `verbatim_doc_comment` on the `Check` variant)
    and in the reference section of `README.md`.
  - `CheckFailure::Unverifiable { doc, cell, reason }` carries the cell and the
    reason; `hickory_cli::unverifiable_message` renders which cell, why, and
    the next step. Since issue #6 the flag exists, so the message names the
    real sequence — un-freeze the cell, `hickory run --cache <doc>`, re-freeze
    — and states that `check` itself has no `--cache` on purpose. The
    `run`-path bail text in `crates/hick-literate/src/lib.rs` names the same
    command; the two no longer disagree. See
    `recordings-are-written-only-when-asked-for.md`.
  - `check_failures` (`crates/hickory-cli/src/lib.rs`) turns every entry of
    `PipelineResult::never_run` into an `Unverifiable` failure, collects
    unmet expectations, and returns early — before the drift loop — when any
    unverifiable entries exist. Expectations are collected *before* that early
    return, which is what lets exit `3` win over exit `2` in a document that
    has both.
  - `hickory check` runs documents in `RunMode::Verify`, which sets
    `PipelineConfig::collect_unverifiable`. In `run_pipeline_live`
    (`crates/hick-literate/src/lib.rs`) that flag turns the two
    no-baseline conditions — a frozen cell with no cache directory, and a
    frozen cell whose recording is missing — into `never_run` entries
    (`NoBaseline::FrozenWithoutCacheDirectory` /
    `NoBaseline::FrozenWithoutRecording`) instead of aborting. `hickory run`
    still uses `RunMode::Execute` and still aborts on the first such cell,
    preserving `docs/guarantees/verification/freeze-is-declared-per-cell.md`.
  - `never_run` is `BTreeMap<CellId, NoBaseline>`, where `CellId` holds
    `Option<String>` container plus source line
    (`crates/hick-literate/src/lib.rs`). `CellId::containerless` exists for a
    cell with no container; nothing in the key assumes one. `render.rs` looks
    cells up with `CellId::exec(...)`.
- Test coverage: `crates/hickory-cli/tests/check_tests.rs` —
  one test per outcome:
  `check_exits_verified_when_nothing_changed` (0),
  `check_exits_drifted_when_a_committed_output_is_out_of_date` (1, with every
  expectation holding, so drift is isolated from a failed claim),
  `check_exits_unverifiable_when_a_cell_has_no_baseline` (2, also asserting
  the message names the cell, the reason, the remedy, and the real
  `hickory run --cache` command rather than the `hick` binary that never
  existed), and
  `check_exits_expectation_failed_when_a_claim_is_false` (3, with the woven
  output committed first so the expectation is the only finding);
  plus one test per precedence pair:
  `a_failed_expectation_outranks_drift`,
  `unverifiable_outranks_drift`, and
  `a_failed_expectation_outranks_unverifiable` (each also asserting both
  findings are still printed).
  `check_reports_unverifiable_when_the_recording_directory_exists_but_the_cell_is_not_in_it`
  and `a_frozen_cell_served_from_its_recording_is_verified_not_unverifiable`
  cover the frozen-cell interaction.
- Caveats requiring LLM review:
  - The server's check path (`apps/server/src/runs.rs`) renders an
    `Unverifiable` failure with the same message, but drives the pipeline
    directly with `collect_unverifiable` left at its default, so it still
    aborts on a frozen cell rather than reporting it. It also reports
    pass/fail rather than an outcome code, so the four-way split is a CLI
    contract only. Worth revisiting when the server grows a cache-aware run
    path.
  - `NoBaseline::NotExecuted` (weave / dry-run) can be produced by the
    pipeline but is not reachable from `hickory check`, which always executes.
    It becomes reachable when agent cells land — an agent cell with no
    recorded session is the other unverifiable case the spec names.
  - The precedence of a failed expectation over unverifiable is a judgement
    call made here (issue #8), not forced by the code: it rests on the claim
    that an unverifiable cell never evaluates an expectation, so the
    expectation finding is genuine. If a future cell kind can produce a
    *partial* expectation result from a missing baseline, that reasoning
    stops holding and the order should be revisited.
