# A Capture Records Values From Inside A Function, On Every Hit

Given `<hick:capture at="file.py:14" of="a, b" when="…" max="N" />` inside a
`<hick:exec>` cell, when the document runs, then each time that line is
reached the same `evaluate` request the interactive debugger issues is sent
for each expression, and the values are woven into the cell's output as a
table ordered by hit index — which `<hick:expect>` can then pin, making a
value *inside a function* something a document can assert about, re-derived on
every run.

Four things this promises beyond "it records values":

- **Bounded.** A capture stops at `max` (default 20) and says in the woven
  output that it stopped, rather than truncating silently.
- **Ordered by hit, not by clock.** A table is stable between runs.
- **Reported, never guessed.** A location the document does not generate, a
  line the debugger will not place a breakpoint on, or an expression that is
  not in scope is written into the document as what happened — not as an
  empty table, which would read as "this never occurred".
- **Degrading.** On a machine with no debug adapter the run still succeeds and
  says what is missing. A document must be readable without `debugpy`.

A malformed capture — no `at`, no `of`, a `max` of zero, an `at` with no line
number — fails the run before any cell executes, with the line it is on.

---

Last LLM verification:
- Date: 2026-08-14
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: parsing in `crates/hick-literate/src/capture.rs` (`collect`,
  `parse_capture`), collected in `run_pipeline_live` by
  `collect_capture_specs` *before* the first cell runs; running in
  `crates/hick-dap/src/capture.rs` (`run` → `drive` → `record`, evaluating
  with DAP's `watch` context, `render` producing the table); the table is
  appended to the cell's recorded output and injected as a transcript entry
  on the cell's source line, which is what puts it in front of
  `<hick:expect>`.
- Test coverage: `crates/hick-dap/tests/live_capture.rs` (six tests against
  real `debugpy`: every hit, conditions, the hit bound, an out-of-scope
  expression, isolation, a bad location);
  `crates/hickory-cli/tests/capture_run.rs` (through the real binary: the
  values reach the woven markdown, and a malformed capture fails first);
  unit tests in both `capture.rs` modules for `at`/`of`/`max` parsing,
  expression splitting, and table rendering.
- Caveat: the live tests skip when no Python adapter is installed.
