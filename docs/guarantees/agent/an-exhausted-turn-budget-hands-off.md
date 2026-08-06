# An Exhausted Turn Budget Hands Off Instead of Failing

Given an agent run that reaches its turn budget without finishing, when the
budget is spent, then the run spends one final call asking the model to stop
and hand off — and returns a successful outcome whose summary is marked
unfinished and carries what changed, what was verified, what is left, and the
next step. It does not return an error.

The old behaviour was `bail!("agent did not finish within N turns")`. By the
time a run hits that line it has usually written files and landed edits: real
work, on disk, that the caller then discards because all it sees is an `Err`.
Measured against the Claude Code sessions on this machine, this is the common
case and not the edge case — the median session used 54 tool-issuing LLM
rounds against a default budget of 20, and 72% exceeded 20.

A handoff also composes with the conversation tree. The summary becomes a
`PriorTurn`, so "keep going" is an ordinary next turn replayed into a fresh
loop rather than a restart from nothing.

The budget itself is still a fixed turn count. A real budget would also bound
tokens, wall clock, and dollars, since twenty turns of `cat` and twenty turns
of full test suites are not the same amount of anything. That is not built;
this guarantee covers only what happens at the edge, not how the edge is
chosen.

---

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
