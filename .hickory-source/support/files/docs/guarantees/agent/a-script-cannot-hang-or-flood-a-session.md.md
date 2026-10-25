<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/agent/a-script-cannot-hang-or-flood-a-session.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `f3c00f17bb79b3d9bca77f6c8062597718c3f1903f31b001c1bf967d61dbc7f6`.

<book:entry path="docs/guarantees/agent/a-script-cannot-hang-or-flood-a-session.md">
# Model snippets cannot hang or flood an agent session indefinitely

Given a model snippet interpreted in Monty, memory, execution duration, callback
count, recursion, sleep and captured output have explicit bounds. Exceeding a
bound produces a failed observation that Python can handle within its iteration
limit. Snippets have no shell or CPython execution fallback. Hosted process
lifetime, frame size and tool requests are independently bounded by Hickory.
Output beyond 64 KiB is refused rather than silently treated as complete.

---

Last LLM implementation review (2026-10-18): partially verified. Requirement,
choices, limitations and executable witnesses are in
[the migration audit](../audits/2026-10-18-python-agent-ownership.md).
Evidence: Python `core.py`, `runtime.py`, `hosted.py`; Rust `python_agent.rs`,
`serve/python_agent.rs`, existing changeset verifier and confinement profile.
Executed checks: see the audit. Human correctness judgment: not recorded.

## Historical evidence for the retired implementation

The following notes predate the migration. Deleted Rust-runtime paths and old
check results do not verify the current implementation.


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
</book:entry>

</book:doc>
