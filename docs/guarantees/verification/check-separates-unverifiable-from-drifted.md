# `hickory check` Separates "Never Established" From "Changed"

Given a document being verified with `hickory check`, when re-derivation is
compared against what is committed, then the command reports exactly one of
three outcomes and exits with that outcome's own code:

- **verified** (exit `0`) — re-derivation matches what is committed.
- **drifted** (exit `1`) — something changed: an unmet `<hick:expect>`, a
  committed output that no longer reproduces, or a stale `<hick:transform>`
  passage.
- **unverifiable** (exit `2`) — at least one cell has **no baseline at all**:
  it neither executed nor was answered from a recording, so there was nothing
  for re-derivation to be compared against.

And:

- A cell served from a **recording** (`freeze="true"`, or a run-wide freeze)
  is **verified, not unverifiable** — a recording is a baseline. Only a cell
  with no recording *and* no execution is unverifiable.
- When a document has both unverifiable cells and drift, the outcome is
  **unverifiable**, and drift comparison for that document is skipped: a cell
  that could not run contributes nothing to the woven output, so the drift it
  would produce is a consequence of the missing baseline rather than a finding.
  Expectations are unaffected — an unverifiable cell never evaluates one.
- The unverifiable report names **which cell** (document, source line, and
  container when it has one), **why it has no baseline**, and **what to do**,
  per `.instructions/user-facing-errors.md`. The remedy must name only flags
  the shipped binary actually accepts.
- A cell is identified by a key that does **not** assume a container, so a
  future agent cell (`docs/specs/freeform/agent-cells.md`), which has none, can
  be reported the same way.

Rationale: drift means someone changed something; unverifiable means nothing
was ever established. CI must be able to respond differently. Conflating them
is how "we have verification" quietly becomes "we have verification for the
parts that ran."

---

Last LLM verification:
- Date: 2026-08-09
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence:
  - `CheckOutcome` and `CheckOutcome::exit_code` in
    `crates/hickory-cli/src/lib.rs` define the three outcomes and their codes
    (`0` / `1` / `2`); `check_outcome` takes the maximum, and `CheckOutcome`
    derives `Ord` in the order `Verified < Drifted < Unverifiable`, which is
    what makes unverifiable outrank drift. `cmd_check` in
    `crates/hickory-cli/src/main.rs` returns `ExitCode::from(worst.exit_code())`
    and prints a distinct summary line per outcome. The table is repeated in
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
    `PipelineResult::never_run` into an `Unverifiable` failure and returns
    early — before the drift loop — when any exist.
  - `hickory check` runs documents in `RunMode::Verify`, which sets
    `PipelineConfig::collect_unverifiable`. In `run_pipeline_live`
    (`crates/hick-literate/src/lib.rs`) that flag turns the two
    no-baseline conditions — a frozen cell with no cache directory, and a
    frozen cell whose recording is missing — into `never_run` entries
    (`NoBaseline::FrozenWithoutCacheDirectory` /
    `NoBaseline::FrozenWithoutRecording`) instead of aborting. `hickory run`
    still uses `RunMode::Execute` and still aborts on the first such cell,
    preserving `docs/guarantees/verification/freeze-is-declared-per-cell.md`.
  - `never_run` is now `BTreeMap<CellId, NoBaseline>`, where `CellId` holds
    `Option<String>` container plus source line
    (`crates/hick-literate/src/lib.rs`). `CellId::containerless` exists for a
    cell with no container; nothing in the key assumes one. `render.rs` looks
    cells up with `CellId::exec(...)`.
- Test coverage: `crates/hickory-cli/tests/check_tests.rs` —
  `check_exits_verified_when_nothing_changed`,
  `check_exits_drifted_when_something_changed`,
  `check_exits_unverifiable_when_a_cell_has_no_baseline` (also asserts the
  message names the cell, the reason, the remedy, and the real
  `hickory run --cache` command rather than the `hick` binary that never
  existed),
  `check_reports_unverifiable_when_the_recording_directory_exists_but_the_cell_is_not_in_it`,
  and `a_frozen_cell_served_from_its_recording_is_verified_not_unverifiable`
  (the frozen-cell interaction).
- Caveats requiring LLM review:
  - The server's check path (`apps/server/src/runs.rs`) renders an
    `Unverifiable` failure with the same message, but drives the pipeline
    directly with `collect_unverifiable` left at its default, so it still
    aborts on a frozen cell rather than reporting it. Worth revisiting when
    the server grows a cache-aware run path.
  - `NoBaseline::NotExecuted` (weave / dry-run) can be produced by the
    pipeline but is not reachable from `hickory check`, which always executes.
    It becomes reachable when agent cells land — an agent cell with no
    recorded session is the other unverifiable case the spec names.
