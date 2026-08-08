# Tier ladder — state machine spec

Detail pulled out of `SKILL.md` to keep it scannable. Load this when
implementing `test-tier-status`/`test-tier-check`/`test-tier-confirm` or when
a question comes up that the workflow steps didn't answer directly.

## `test-tiers.yaml` fields

One entry per test/flow, under `tests:`. Fields:

| Field | Values | Meaning |
|-------|--------|---------|
| `tier` | `eyeball` \| `watched` \| `automated` | Current trust level |
| `coverage_report` | path (lcov/cobertura/JSON) | Line-coverage report from this test's last run with coverage instrumentation enabled. Derived by actually running the test, not curated — see `SKILL.md`'s note on why code-review-graph's own `TESTED_BY` edges don't reach E2E flows |
| `consecutive_passes` | integer, default 0 | Confirmed clean runs at the current tier since the last demotion or promotion |
| `last_verified_commit` | git SHA \| `null` | The commit this test's current tier was last confirmed against — the base for the next blast-radius check |
| `history` | list of `{date, event, note}` | Dated log of every verification, demotion, and promotion for this test |

Each verification appends a `history` entry, e.g.:

```yaml
history:
  - date: "2026-08-03"
    event: "watched run confirmed pass"
    note: "Confirmed by nate@loumtechnologies.com. Reviewed the trace, dashboard showed usage metrics for the tunnel as expected."
  - date: "2026-08-10"
    event: "demoted: automated -> watched"
    note: "cloud/api/src/plan_limits.rs:112-140 changed in a1b2c3d..f4e5d6c, which this test's coverage_report shows it exercises."
```

`test-tier-check` and `test-tier-confirm` write these entries directly (as
plain YAML edits), then commit `test-tiers.yaml` — git history/blame is the
audit trail for who ran which recipe and when, no signature needed in the
text itself.

## Demotion algorithm (`test-tier-check`, run in CI on every push)

```
for each tracked test:
    changed = blast_radius(base=test.last_verified_commit, head=HEAD)
    # blast_radius: code-review-graph detect-changes --base <base> if available
    # and the changed files' languages are supported (returns changed
    # files/functions/line-ranges); else `git diff <base>..HEAD` parsed for
    # changed line ranges per file

    hit = coverage_overlaps(test.coverage_report, changed)
    # coverage_overlaps: does any line the test's coverage report marks as
    # executed fall inside a changed line range? File-level overlap alone
    # is NOT sufficient — two tests can share a file and touch different
    # lines, and only the one that actually covers the changed lines should
    # demote.
    just_failed = test.last_recorded_result == "fail"

    if hit or just_failed:
        demote_one_tier(test)          # floor at eyeball — no-op if already there
        test.consecutive_passes = 0
        append_history_entry(test, event="demoted", note=(changed_summary) if hit else "last run failed")
```

Run this **before** executing any test's command for the cycle, so a test
that gets demoted this cycle is graded (and gated) at its *new*, lower tier
immediately — not next cycle.

## Execution + promotion algorithm

```
for each tracked test:
    if test.tier == "automated":
        result = run_with_coverage(test.run.automated)   # refreshes test.coverage_report
        record_result(test, result)
        if result == pass:
            test.consecutive_passes += 1
            if test.consecutive_passes >= promote_after:
                promote_one_tier(test)   # ceiling at automated — no-op if already there
                test.consecutive_passes = 0
        else:
            test.consecutive_passes = 0   # failure alone doesn't demote here — the
                                           # NEXT cycle's demotion pass (above) does,
                                           # so this cycle's failure is still visible
                                           # as a normal CI red first
        test.last_verified_commit = HEAD

    elif test.tier == "watched":
        result = run_with_coverage(test.run.watched)
        upload_artifacts(result)          # logs/trace/screenshots for the human gate
        # STOP here for this test this cycle — do not touch consecutive_passes
        # or last_verified_commit until a human runs test-tier-confirm.
        # The CI workflow's `await-confirmation` job (references/github-actions-gate.md)
        # is what turns an unattended `result` into a counted pass.
        report_needs_human(test)

    else:  # eyeball
        report_needs_human(test)          # nothing to run unattended at all
```

`test-tier-confirm <id> pass|fail "<note>"` is the only path that can turn a
`watched` run into a counted pass:

```
record_result(test, verdict)
append_history_entry(test, event=("confirmed pass" if verdict == pass else "confirmed fail"), note=note)
test.last_verified_commit = HEAD
if verdict == pass:
    test.consecutive_passes += 1
    if test.consecutive_passes >= promote_after:
        promote_one_tier(test)
        test.consecutive_passes = 0
else:
    test.consecutive_passes = 0
    # a failed watched/eyeball run does NOT demote further than where it
    # already is — eyeball is the floor, and a watched test staying watched
    # after a confirmed failure is correct (it needs fixing, not demoting)
```

## Why demotion and promotion aren't symmetric

Promotion requires a human-confirmed streak; demotion can happen from a
purely mechanical signal (coverage/blast-radius overlap or a bare CI
failure). This asymmetry is intentional: earning trust should require a
person to have actually looked at the result at least once per streak-step,
but losing trust shouldn't wait on a person noticing — the whole point of
automatic demotion is to catch the case where nobody would have thought to
re-check that test.
