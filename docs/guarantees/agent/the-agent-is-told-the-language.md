# The Built-In Agent Is Told The Document Language, And Where Its Scripts Run

Given a `hick agent` run with a primary document, when the tool set is
enabled, then the system prompt carries a reference of the elements a
document may contain — container, volume, exec with a child expect, file,
copy, paste by `#id`/`.class`, upstream, transcript turns, transform, claim —
and two rules that cost a real run: an exec's output is not a fragment, and
the agent's scripts run in a scratch workspace that does not see the project,
so the real inputs are read by `verify`, never recreated.

The reason is a recorded session. Asked to analyse a CSV that sat beside the
document, the agent with no grammar in its prompt invented attributes
(`hick:paste from=… from-line=…`, `hick:exec cmd="python"`), put the
expectation outside the cell, and — finding no CSV in its scratch directory —
**wrote a fake one and pinned invented numbers** as findings
(`examples/receipts/hick-agent/sessions/20260822-125634-…`). With the
reference in the prompt the same task was structured correctly on the next
run; the remaining failure was the expectation's leading newline, which the
mismatch message now names
(`crates/hick-literate/src/expect.rs`, "begins with an empty line").

The prompt also names `read_file`: the agent's window onto project files
that are not documents, read-only and recorded
(`context-provenance-is-derived-from-the-session.md`). The scratch workspace
still cannot see the project — that is what keeps the document the only write
path — but the agent no longer has a reason to invent what it cannot find.

## Boundary

The reference is a crib, not the parser: an element it does not list still
parses.

---

Last LLM verification:
- Date: 2026-08-22
- Reviewer: Claude Fable 5
- Result: verified (implemented and reviewed in the same change)
- Evidence: `TOOLS_SYSTEM_PROMPT` in `crates/hickory-agent/src/protocol.rs`
  ("The document language"); `diff_detail` in
  `crates/hick-literate/src/expect.rs` and its test
  `a_leading_newline_in_the_expectation_is_named`.
- Sessions: `examples/receipts/hick-agent/sessions/` — the first two without
  the crib (30 turns, failed), the third with it (25 turns, verify passed),
  `fix.hick` in 16 turns, `message.hick` in 4.
