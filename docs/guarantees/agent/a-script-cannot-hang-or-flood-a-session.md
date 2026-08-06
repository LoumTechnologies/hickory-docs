# A Script Cannot Hang or Flood a Session

Given an agent script that never terminates, when it exceeds the session's
script timeout, then it is killed inside its own container and the observation
says so in words. Given a script that writes far more output than an
observation can carry, when it finishes, then the head and the tail are kept,
the middle is dropped, and the marker states how many bytes were omitted.

An agent's script is written by a model, so "it terminates" and "it prints a
reasonable amount" are assumptions. Neither was enforced. A `cargo test` that
blocks on a prompt held the run forever with no upper bound; one `grep -r`
over a large repository returned megabytes that were pasted verbatim into the
next request, buying nothing and crowding out everything else.

Two details are deliberate:

**The kill happens in the container**, via `timeout -k 5`, not by racing an
outer future against the call. Racing returns control to the loop while the
runaway process keeps holding the container's CPU and files — the agent
believes it moved on while the machine underneath it did not. `timeout` ends
the process; `-k` follows with SIGKILL for anything ignoring SIGTERM. An outer
`tokio::time::timeout`, set generously above the inner one, remains as a
backstop for the executor itself hanging (a lost connection to a remote
microVM never reaches `timeout` in the guest).

**Truncation keeps both ends.** The head shows what the script was doing; the
tail holds the error and the exit status. Head-only truncation — the obvious
implementation — discards precisely the part the model needs. The byte count
in the marker lets the model tell a truncated observation from a complete one
and narrow its next command rather than concluding it has seen everything.

---

Last LLM verification:
- Date: 2026-08-06
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-agent/src/script.rs` — `ScriptLimits` (default 600s
  and 16 KiB) is carried on `AgentConfig::script_limits`; `run_script` wraps
  the language's run command in `timeout -k 5 <secs>`, labels exit code 124 as
  the agent script timeout, and passes both stdout and stderr through
  `clamp_output`, which slices on char boundaries so multi-byte output cannot
  panic.
- Test coverage: `script.rs` `limit_tests` —
  `a_script_that_never_finishes_is_killed_and_labelled` runs `sleep 30` under a
  1s limit and asserts it returns quickly with exit 124 and a named reason;
  `a_flood_of_output_is_clamped_before_it_reaches_the_model` runs a ~1.6 MB
  emitter under a 4 KiB limit and asserts the observation stays small and says
  it was truncated; plus head/tail retention and the char-boundary case.
