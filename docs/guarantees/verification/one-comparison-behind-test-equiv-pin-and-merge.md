# One Comparison Behind Test, Equiv, The Pin, And The Merge Check

Given two sets of woven outputs — a document's against what is on disk
(`hick test`), one document's against another's (`hick equiv`), a document's
against its pinned baseline (the refactor pin in the app), a document with
its recordings inside it against the cache-backed weave (the ingest gate),
or a merged document against its parents (the merge driver, through
`hick test`) — when they are compared, then it is one function,
`hick_literate::equiv::compare_outputs`, and one report shape: per file,
*only in the first*, *only in the second*, or *content differs* with the
one sentence every comparison ends with — where the bytes first part company
and how the lengths compare (`describe_difference`), followed by the
line-by-line difference.

Five comparisons that each asked "would these two weaves produce the same
bytes?" had grown three implementations and three vocabularies. Axis 3 of
`docs/specs/freeform/three-axes.md`: agreement is one question, so it is one
answer.

## Boundary

`hick test` still skips volatile outputs and still reports a missing file as
missing rather than as a diff; those are decisions about what to compare,
not about how.

---

Last LLM verification:
- Date: 2026-09-03
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hick-literate/src/equiv.rs` (`compare_outputs`,
  `describe_difference`, `format_diff`); callers in
  `crates/hickory-cli/src/lib.rs` (`check_failures`, drift),
  `crates/hickory-cli/src/main.rs` (`cmd_equiv`),
  `crates/hickory-cli/src/serve/refactor.rs` (the pin), and
  `crates/hickory-cli/src/ingest_recording.rs` (the gate). The merge driver
  delegates to `hick test`.
- Test coverage: `crates/hick-literate/src/equiv.rs::describe_difference_tests`
  (moved from the CLI with the function); `crates/hickory-cli/tests/test_command_tests.rs`
  (drift verdicts); `crates/hickory-cli/tests/round_trip.rs` (the gate).
