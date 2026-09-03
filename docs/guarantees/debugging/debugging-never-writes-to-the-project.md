# Debugging A Document Never Writes To The Project

Given a document whose cells write files, when it is debugged — interactively
from the desktop app or the MCP tools, or non-interactively by a
`<hick:capture>` on a run — then nothing the debugger does reaches the
repository: no generated file, no output the debuggee wrote, no change to the
document, and no change to a recorded transcript.

The program under a debugger is always woven into a **scratch directory** and
launched there, and that directory is deleted when the session ends. This is
what makes the property architectural rather than a matter of care: a session
has nowhere to write *to*. The debug channel (`0x03`) carries no edit
operation either, so an app that is stepping cannot also be editing.

Three things are deliberately outside the guarantee:

- **A plain file runs in place.** A file that is not a document —
  `src/main.rs`, `app.py` — is debugged as itself, in its own project, with
  the person's own build (`a-plain-file-has-the-same-debugger.md`). It has
  nothing woven to protect and no transcript to keep honest; it is their
  program in their checkout, and what it writes is what it would write
  under `cargo run`. The debug channel still carries no edit operation, so
  the *pane* cannot change the file while stepping; the *program* may do
  whatever the program does.

- **Evaluating an expression can change the debugged program.** `lines.pop()`
  really pops. That is the debuggee's own business, and it dies with the
  session. It is why a `<hick:capture>` is sent with DAP's `watch` context
  instead — the context adapters treat as repeatable.
- **Promotion writes, because a person asked it to.** Turning a value seen at
  a breakpoint into a `<hick:capture>` goes through the ordinary edit path,
  undoable and reviewable like anything typed.

---

Last LLM verification:
- Date: 2026-08-14
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `hick_dap::program::weave_into` writes a document's generated
  files into a caller-supplied directory, and both callers supply a
  `tempfile::TempDir`: `crates/hickory-cli/src/debug_sessions.rs`
  (`Registry::start`, whose `Live` owns the `TempDir` so it is removed with
  the session) and `crates/hick-dap/src/capture.rs` (`run`, whose scratch dir
  is dropped when the run returns). `Launch::cwd` is that directory in both
  cases. The socket bridge in
  `crates/hickory-cli/src/serve/debug_bridge.rs` defines the whole `Request`
  enum — start, breakpoints, state, eval, step, jump, run-to, children,
  set-variable, stop — and none of them edits anything.
- Test coverage:
  `crates/hickory-cli/tests/debug_desktop.rs::nothing_a_debugger_does_touches_the_project`
  (drives the real socket through a document whose program writes
  `evidence.txt`, then asserts the project directory is unchanged
  byte-for-byte);
  `crates/hick-dap/tests/live_capture.rs::a_capture_run_writes_nothing_into_the_project`;
  `crates/hickory-cli/tests/capture_run.rs::a_captured_run_leaves_the_project_alone`
  (through the real `hick run` binary — the project holds the document, its
  woven output, and the file the document generates, and nothing else).
- Caveat: the live tests skip loudly when no Python debug adapter is
  installed, so a machine without `debugpy` verifies the bridge's shape but
  not the running program. CI installs one.
