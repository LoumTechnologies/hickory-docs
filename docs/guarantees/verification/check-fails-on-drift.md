# `hickory check` Fails When Documented Output Drifts From Real Output

Given a document containing `<hick:expect>` blocks, when `hickory check`
re-executes the document and any produced output no longer satisfies its
expectation (`exact` byte equality, or `regex-lines` full-line regex match),
then the command exits non-zero and reports the failing block with its source
span — documentation drift is a build failure, never a warning.

Drift is exit code `1` specifically; a cell that was never verified against
anything is a *different* outcome with its own code. See
`check-separates-unverifiable-from-drifted.md`.

---

Last LLM verification:
- Date: 2026-08-05
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence: `hickory check` (crates/hickory-cli) re-executes each document
  through the `Executor` boundary; `hick-literate/src/expect.rs` evaluates
  every `<hick:expect>` (`exact` byte equality / `regex-lines` anchored
  full-line regexes covering all output lines) and `check_failures` in
  `crates/hickory-cli/src/lib.rs` turns any unmet expectation — or drift
  between produced outputs and committed files — into a non-zero exit,
  reporting doc path, block source line, byte span, and expected vs actual.
  Verified live: `just verify examples/` exits 0 on both shipped examples;
  `hickory check crates/hickory-cli/tests/fixtures/drifted-tour.hick`
  (deliberately drifted copy: `apple 12` → `apple 13`) exits 1 listing the
  failing block at line 45 with both outputs.
- Test coverage: `crates/hickory-cli/tests/check_tests.rs` —
  `check_fails_on_drifted_expectation` (the guarantee's test), plus
  `check_fails_on_committed_output_drift`,
  `check_fails_on_drifted_fixture_copy_of_shipped_example`,
  `check_passes_on_matching_expectations`,
  `regex_lines_expectations_pass_and_fail`, and
  `run_succeeds_and_records_failed_expectation_without_failing`
  (run records but never fails).
