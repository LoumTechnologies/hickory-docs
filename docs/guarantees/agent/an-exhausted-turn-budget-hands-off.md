# An exhausted Python iteration budget retains incomplete work

Given a Python run that spends its configured iteration budget without current
accepted completion, the Python graph stops and reports an error. Hickory keeps
actual applied writes, proposals, model calls and tool evidence in the session.
It does not mark the unverified work successful or make a hidden Rust handoff
model call. A later turn can use the retained session to continue deliberately.

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
- Evidence: `crates/hickory-agent/src/react_loop.rs` — after the `for turn in
  0..config.max_turns` loop falls through, a wrap-up message is appended to
  the history and one `stream_completion` runs; its usage is added to
  `total_usage`, the summary is prefixed `[unfinished — stopped after N
  turns]`, recorded as `SessionEvent::Assistant` and `SessionEvent::End`, and
  returned as a normal `AgentOutcome`.
- Test coverage: `crates/hickory-agent/tests/turn_budget.rs` —
  `an_exhausted_budget_returns_a_handoff_instead_of_failing` drives a scripted
  model that never says done against `max_turns = 2`, and asserts the run
  succeeds, the summary is marked unfinished, and the model's handoff text
  reaches both the outcome and the session document.
