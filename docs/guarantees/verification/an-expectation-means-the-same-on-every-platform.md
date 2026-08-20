# An Expectation Means The Same Thing On Every Platform

Given a document whose `<hick:exec>` cell carries an `<hick:expect>`, when the
document is run on any supported platform, then the cell's captured output is
recorded with `\n` line endings — a `\r\n` emitted by the shell, the program,
or the container becomes `\n` at capture, before it reaches the transcript
entry, the returned stdout, the woven `.md`, or a `.hick-cache` recording. A
document that passes on macOS therefore passes on Windows, and a recording made
on one serves a run on the other.

`match="exact"` is still byte-for-byte. It is not relaxed, weakened, or made
line-ending-insensitive: what changed is that the bytes it compares against are
portable. `exact_is_byte_exact_about_trailing_newline` in
`crates/hick-literate/src/expect.rs` still holds — a missing final newline is
still a failure.

Only `\r\n` is rewritten. A lone `\r` is left alone, because it is how a
progress bar or a spinner repaints one line, and rewriting it would turn one
line of woven output into hundreds. (The canopy executor is stricter and drops
every `\r`: its stream is a pty, where both forms are cursor management rather
than content.)

The one thing this makes unsayable, stated rather than hidden: a cell can no
longer assert *through stdout* that a program emits DOS line endings. A cell
that needs to claim that must write the output to a file and report the bytes
(`od -c`, `xxd`), which is a sturdier test of it anyway.

---

Last LLM verification:
- Date: 2026-08-20
- Reviewer: Claude (Opus 5)
- Result: verified on macOS 15.7.7 (Windows behaviour reasoned from the
  measurement in issue #18, not re-measured here — see caveat)
- Evidence: the rule and its rationale live in one place,
  `crates/hickory-executor/src/capture.rs`
  (`normalize_captured_newlines`, plus `CapturedStream` for chunked capture).
  It is applied at the two places in the workspace where bytes from a host
  process become recorded text: `LocalExecutor::run_command_as`
  (`crates/hickory-executor/src/lib.rs`) — for the aggregate stdout/stderr and
  for each streamed transcript event — and `DockerExecutor::run_command`
  (`crates/hickory-executor-docker/src/lib.rs`). `SandboxedExecutor` needs no
  change of its own: every one of its `execute*` methods delegates to
  `LocalExecutor::execute_argv_as`, which is the same function. The canopy
  executor already satisfied it by a stricter route —
  `hickory_executor_canopy::frame::strip_ansi` drops every `\r` before a line
  becomes a `FrameItem::Line`, which is why its contract test has always
  asserted `"hello\n"` against a mock guest that emits `"hello\r\n"`. The
  invariant is stated as part of the `Executor` trait contract on
  `Executor::execute` and on `ExecTranscriptEntry::output`, since a future
  executor that decodes process bytes is the way it would be lost.
  `hick-literate/src/expect.rs` is deliberately untouched.
- Test coverage:
  `crates/hickory-executor/src/capture.rs` unit tests — `crlf_becomes_lf`,
  `a_lone_carriage_return_is_left_alone`,
  `a_crlf_split_across_chunks_is_still_normalised` (an 8 KiB read boundary
  landing between the `\r` and the `\n`),
  `a_carriage_return_at_the_very_end_survives`,
  `an_empty_chunk_does_not_release_a_held_back_cr`;
  `crates/hickory-executor/src/lib.rs::captured_output_is_recorded_with_lf_line_endings`,
  which runs a cell that really emits CRLF (`echo one& echo two` under cmd,
  `printf 'one\r\ntwo\r\n'` under sh) and asserts the returned stdout, the
  transcript entry, and every streamed event are LF;
  `crates/hickory-cli/tests/test_command_tests.rs` end to end through the
  shipped binary — every document there carries `match="exact"` and is now
  written in each platform's own shell (issue #17's helper pattern), so the
  whole suite is a per-platform check of this guarantee.
- Caveat requiring review: the Windows half is verified by CI, not by this
  review — the reviewer has no Windows machine and the repo's single VM is
  under human coordination. The Windows job in `.github/workflows/ci.yml`
  running `cargo test -p hickory-cli --test test_command_tests` is what turns
  this from reasoning into evidence; until that line exists the Windows claim
  rests on the shipped-binary measurement recorded in issue #18.
- Caveat requiring review: an existing `.hick-cache` recording written by an
  older binary on Windows holds CRLF and will not match a cell re-run under
  this version. A `freeze="true"` cell whose recording was made that way
  reports drift once — `hick run` rewrites the recording, and it agrees
  thereafter. No recording of that kind is committed to this repository (the
  only recordings here are made by tests in temp directories), so nothing in
  the tree is stale.
