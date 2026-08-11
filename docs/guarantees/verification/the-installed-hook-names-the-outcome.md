# The Pre-Commit Hook `hickory init` Installs Names Which Outcome Blocked The Commit

Given a repository set up by `hickory init`, when the installed pre-commit
hook blocks a commit because `hickory test` exited non-zero, then the message
names the specific outcome that occurred — drift (exit `1`), not verified
(exit `2`), or a failed expectation (exit `3`) — and states the fix for that
outcome and only that outcome.

Specifically:

- exit `1` says DRIFT and tells the author to re-run `hickory run <doc>` (or
  `hickory refresh <doc>` for a stale `hick:transform`) and commit the result;
- exit `2` says NOT VERIFIED and tells the author to record a baseline by
  running `hickory run <doc>` — a cell declaring `freeze="true"` runs once
  there and records itself — or stop freezing the cell;
- exit `3` says FAILED EXPECTATION and explicitly tells the author **not** to
  regenerate it away, because the document claims something untrue of its own
  output and a human has to decide whether the claim or the code is wrong.

When several documents fail, the hook reports the strongest outcome, using the
same precedence the CLI uses: verified(0) < drifted(1) < unverifiable(2) <
expectation failed(3).

This matters because the three outcomes have three different fixes, and one of
them is the opposite of another: drift is fixed *by* regenerating, a false
claim is destroyed by regenerating. A hook that says "documentation drift" for
all of them sends the author to the wrong fix for two cases out of three.

---

Last LLM verification:
- Date: 2026-08-10
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `HOOK_BODY` in `crates/hickory-cli/src/init.rs` captures each
  document's exit code (`hickory test "$hick_doc" || hick_code=$?`, written
  in that form so the block is safe to append to a user hook running under
  `set -e`), keeps the numeric maximum in `hick_worst`, and branches on it in
  a `case` with one message per outcome plus a catch-all for an unexpected
  code. The numeric maximum is the same precedence `CheckOutcome`'s `Ord`
  gives the CLI (`crates/hickory-cli/src/lib.rs`), because the exit codes are
  assigned in that order. Verified live in a scratch repository: a hook
  containing a stale managed block was rewritten by `hickory init` to the new
  body, and the surrounding user content was left untouched.
- Test coverage: `crates/hickory-cli/tests/init_tests.rs` —
  `hook_names_drift_when_a_committed_output_is_stale` (exit `1`: asserts the
  message says DRIFT and does *not* say FAILED EXPECTATION) and
  `hook_passes_with_clean_doc_and_fails_on_a_false_claim` (exit `3`: asserts
  the message names the failed-expectation outcome and no longer says
  "documentation drift"). `init_installs_hook_idempotently` covers the
  sentinel-delimited rewrite that gets an existing repository onto the new
  wording.
- Caveat: exit `2` (not verified) has no hook-level test — it needs a frozen
  cell with no recording, which the other exit-`2` coverage exercises at the
  CLI level (`test_freeze_reports_an_unrecorded_cell_as_unverifiable` in
  `crates/hickory-cli/tests/cache_flags.rs`). The hook's `case` arm for it is
  reviewed by LLM only; the shared branch structure is exercised by the two
  tested arms.
