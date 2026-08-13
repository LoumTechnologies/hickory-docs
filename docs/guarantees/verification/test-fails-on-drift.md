# `hick test` Fails When Documented Output Drifts From Real Output

Given a document containing `<hick:expect>` blocks, when `hick test`
re-executes the document and any produced output no longer satisfies its
expectation (`exact` byte equality, or `regex-lines` full-line regex match),
then the command exits non-zero and reports the failing block with its source
span — documentation drift is a build failure, never a warning.

An unmet expectation is exit code `3` specifically — the document claims
something untrue of its own output, which is a different problem from a
committed file that is merely out of date (exit `1`), and from a cell that was
never verified against anything (exit `2`). See
`test-separates-unverifiable-from-drifted.md` for all four codes and their
precedence.

---

Last LLM verification:
- Date: 2026-08-10
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `hick test` (crates/hickory-cli) re-executes each document
  through the `Executor` boundary; `hick-literate/src/expect.rs` evaluates
  every `<hick:expect>` (`exact` byte equality / `regex-lines` anchored
  full-line regexes covering all output lines) and `check_failures` in
  `crates/hickory-cli/src/lib.rs` turns any unmet expectation — or drift
  between produced outputs and committed files — into a non-zero exit,
  reporting doc path, block source line, byte span, and expected vs actual.
  Verified live: `just verify examples/` exits 0 on both shipped examples;
  `hick test crates/hickory-cli/tests/fixtures/drifted-tour.hick`
  (deliberately drifted copy: `apple 12` → `apple 13`) exits non-zero listing
  the failing block at line 45 with both outputs — exit `3` since issue #8
  gave a failed expectation its own code. Re-verified after issue #9 renamed
  the subcommand from `check` to `test` with no alias: `hick check` now
  exits as an unrecognized subcommand (`the_old_check_subcommand_is_gone`).
- Test coverage: `crates/hickory-cli/tests/test_command_tests.rs` —
  `test_fails_on_drifted_expectation` (the guarantee's test), plus
  `test_fails_on_committed_output_drift`,
  `test_fails_on_drifted_fixture_copy_of_shipped_example`,
  `test_passes_on_matching_expectations`,
  `regex_lines_expectations_pass_and_fail`, and
  `run_succeeds_and_records_failed_expectation_without_failing`
  (run records but never fails).
