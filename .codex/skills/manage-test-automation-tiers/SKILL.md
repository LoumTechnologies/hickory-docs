---
name: manage-test-automation-tiers
description: >
  Grade each test or E2E flow along a trust ladder (eyeball → watched →
  automated) and keep that grade honest as code changes, using real
  line-coverage data (not curated file globs) to know which tests actually
  exercise the lines a diff touched. Use when asked to set up a system where
  fully-manual tests can be promoted to unattended CI checks over time, where
  a code change should "downgrade" tests back to human eyeballs until
  re-proven, or to wire code-review-graph's blast-radius analysis into CI
  gating decisions. Complements $audit-feature-test-coverage (builds the E2E
  coverage this skill grades) and $plan-deploy-shared (GitHub Environment/
  required-reviewer conventions this skill's watched tier reuses). State
  lives in a single YAML file in the target repo; uses $just for the
  recurring commands.
---

# Manage Test Automation Tiers

## Goal

Not every test deserves the same amount of trust forever. A test a human just
wrote and eyeballed is not yet safe to run unattended in CI; a test that's
passed unattended dozens of times against unchanged code is safe to trust
without a human watching every run; and a test whose covered code just
changed — no matter how long it's been green — needs a human to look again
before it goes back to being trusted blindly. This skill sets up that ladder
(`eyeball → watched → automated`) for a target project's tests/flows, and
keeps promotions and demotions driven by real signals — actual code-coverage
data from running the tests, and code-review-graph's blast-radius analysis of
what a diff changed — rather than by nobody ever revisiting the question.

This skill does **not** invent a way to run tests. It orchestrates:

1. **The target project's own test commands, run with code coverage
   instrumentation enabled** — a unit runner, a Playwright/Cypress suite, or
   a manual walkthrough tool (e.g. the guided C# tool built for
   `portzero-full-example`, or whatever `$audit-feature-test-coverage` set
   up), each run through whatever coverage tool the project's language
   already has (`pytest --cov`, `go test -cover`, `cargo llvm-cov`,
   `nyc`/`c8` for JS, etc.). The coverage report from a given test's run is
   the source of truth for which lines it actually exercises — nobody has to
   curate a list of "files this test covers" by hand, and the data stays
   accurate even for E2E flows that drive the app over HTTP/browser (where
   code-review-graph's own static `TESTED_BY` call-graph edges don't reach,
   since those only connect same-process unit tests to the code they call
   directly).
2. **`code-review-graph`** (assumed installed — `~/.local/bin/code-review-graph`,
   also usable as an MCP server) for blast-radius computation: which
   files/functions/lines a diff touches. Cross-referencing that against each
   test's last-recorded coverage report is what decides whether a change
   actually threatens a given test's trust, not just whether the change
   landed in the same file.
3. **A single state file, `test-tiers.yaml`**, checked into the target repo's
   root — holds every tracked test's current tier, its coverage report
   pointer, its consecutive-pass count, and a dated history log. No external
   issue tracker, ticketing system, or milestone is involved.

## The tier ladder

| Tier | Runs unattended? | Counts without a human? |
|------|-------------------|--------------------------|
| `eyeball` (fully interactive) | No — a human runs it by hand | No — the human's verdict *is* the result |
| `watched` (watch-only) | Yes | No — CI runs it, but a human must confirm the result before it counts |
| `automated` (fully automated) | Yes | Yes — a normal required CI check, no human touch |

**Promotion**: after `promote_after` (default 3) consecutive clean, human-
confirmed runs at the current tier, a test moves up one step. An unattended
`watched` pass that nobody has confirmed **never** counts toward promotion —
only a confirmed one does.

**Demotion**: one tier down at a time (floor at `eyeball` — already-manual
tests can't demote further), triggered by **either**:
- A **coverage/blast-radius hit**: a code-review-graph (or, degraded, plain
  diff) run says a change touched lines that the test's last-recorded
  coverage report shows it actually exercised. File-level overlap alone is
  not enough — two tests can live in the same file and touch different lines,
  and only the one whose coverage report actually hit the changed lines
  should demote.
- A **failed last run**: regardless of coverage overlap, a test that just
  broke doesn't get to go straight back to trusted-automated on its next
  green run — a human needs to confirm the fix first.

Either trigger resets `consecutive_passes` to 0 and appends a history entry
explaining why (which lines hit, or what failed).

## Workflow

1. **Inventory the project's existing tests/flows.** For each one worth
   tracking (typically: every E2E/manual flow `$audit-feature-test-coverage`
   set up, plus any integration suites the team wants graduated over time),
   add an entry to `test-tiers.yaml` (scaffolded in step 3). Start every new
   entry's `tier` honestly — usually `eyeball`, never `automated` just
   because the command happens to run unattended today.

2. **Run each test with coverage instrumentation enabled** to produce its
   line-coverage report (lcov, cobertura XML, JSON — whatever the project's
   coverage tool emits) and store the report path/artifact reference against
   that test in `test-tiers.yaml`. This is what stands in for a manually
   curated "covers" list: the report says, precisely, which lines that test
   run actually exercised. Re-run this step whenever a test's own command or
   the surrounding code changes enough that its old coverage report might be
   stale.

3. **Confirm blast-radius tooling.** Run `code-review-graph status` in the
   target repo. If it's missing, broken, or the changed files are in a
   language it doesn't parse, don't block on it — fall back to `git diff
   <last_verified_commit>..HEAD` (full diff, not just `--name-only`, since
   the demotion check needs changed *line* ranges to intersect against
   coverage data) for the demotion check. A coarser signal beats no signal;
   note the degraded mode in the report rather than silently pretending it's
   using the graph.

4. **Scaffold `test-tiers.yaml`** at the target repo's root:
   ```yaml
   promote_after: 3
   tests:
     - id: audit-log-gated
       tier: eyeball
       consecutive_passes: 0
       last_verified_commit: null
       coverage_report: .test-tiers/coverage/audit-log-gated.lcov
       run:
         eyeball: "just test-team --step audit-log-gated"
         watched: "just test-team --step audit-log-gated"
         automated: "just test-team --step audit-log-gated"
       history: []
   ```
   `run` commands may be identical across tiers (as above, when the
   underlying tool already knows how to run unattended-with-optional-confirm)
   or different (e.g. a Playwright spec for `automated`/`watched` and a
   written checklist doc for `eyeball`). Either is fine — the tier only
   changes who has to bless the result, not necessarily the command. Every
   run recorded against a test — pass, fail, or human confirmation — appends
   a dated entry to that test's `history` list in place, e.g.:
   ```yaml
       history:
         - date: "2026-08-03"
           event: "watched run confirmed pass"
           note: "Reviewed the trace, dashboard showed usage metrics as expected."
         - date: "2026-08-10"
           event: "demoted: automated -> watched"
           note: "coverage/blast-radius hit on plan_limits.rs:112-140"
   ```

5. **Scaffold the `just` recipes** (per `$just`; thin wrappers around a small
   helper script in the target repo's primary backend language — do not
   introduce a second language just for this):
   - `just test-tier-status` — prints a checklist table: `[x]`/`[ ]`, id,
     tier, last verified commit, consecutive passes, blocked reason if any —
     the list of tests with checkboxes the user asked for, straight from
     `test-tiers.yaml`.
   - `just test-tier-check` — the CI entry point. Computes demotions (step 3's
     blast-radius/diff check, intersected against each test's
     `coverage_report`, plus checking each `automated`-tier test's last
     recorded result), then runs every `automated`-tier test's command with
     coverage enabled (refreshing `coverage_report`), then prints which
     `watched`/`eyeball` tests still need a human this cycle.
   - `just test-tier-confirm <id> pass|fail "<note>"` — records a human's
     eyeball/watched verdict directly into `test-tiers.yaml`: appends a
     history entry, updates `tier`/`consecutive_passes`/`last_verified_commit`,
     and commits the file so the checklist state is durable and diffable in
     git history.

6. **Scaffold the CI workflow** for the `watched` tier — see
   `references/github-actions-gate.md` for the concrete YAML and the
   one-time `test-tier-watch` GitHub Environment setup (required reviewer,
   per `$plan-deploy-shared`'s environment conventions). `automated`-tier tests
   just run as a normal job in the project's existing CI; they need no new
   gate.

7. **Report** the resulting tier checklist (same shape as `test-tier-status`)
   and flag anything degraded (code-review-graph unavailable, a test with no
   `coverage_report` yet, an `eyeball`-tier test that's never been run).

See `references/tier-ladder.md` for the full state-machine spec (field
definitions, exact demotion/promotion pseudocode) if a deeper implementation
detail is needed mid-workflow.

## Guardrails

- Never invent a new way to *run* a test. Call into whatever command the
  project already uses — a unit runner, `$audit-feature-test-coverage`'s
  Playwright suite, or a manual walkthrough tool. This skill only changes who
  has to bless the result, and instruments that run with coverage.
- If `code-review-graph` can't parse a file's language or isn't installed,
  degrade to a plain `git diff` (with line ranges, not just filenames) rather
  than blocking demotion detection entirely, and say so in the report.
- Don't demote on file-level overlap alone when line-level coverage data is
  available — check whether the test's `coverage_report` actually hit the
  changed lines.
- A `watched`-tier run that fails never reaches the human-confirm gate framed
  as a pass — CI failure is still CI failure, independent of the tier system.
- Never let an unattended pass alone promote a `watched` test to
  `automated` — promotion requires the human-confirm step to have actually
  run at least `promote_after` times.
- Demotion only ever moves down; a hit or a failure on a test already at
  `eyeball` is a no-op, not an error.
- Don't retroactively mark a test's starting tier as `automated` just because
  it happens to be unattended today — start honest (usually `eyeball` or
  `watched` for anything newly tracked) and let it earn `automated` through
  real consecutive confirmed passes.
- All state lives in `test-tiers.yaml`, checked into the repo. Don't reach
  for an external issue tracker or ticketing system for this — the whole
  point is a plain, diffable, git-native checklist.
