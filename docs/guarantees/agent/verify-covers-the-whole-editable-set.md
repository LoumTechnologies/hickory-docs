# Verify Covers the Whole Editable Set

Given an agent session whose primary document declares `hick:upstream` edges,
when the agent runs `verify`, then every document in that closure is executed
and its output files are written **in its own directory** — and the result says
how many upstream documents were re-woven. A `PASS` means the whole chain is
consistent, not just the document in front of the agent.

Verification scope must equal edit scope. Once the agent could edit any
document in the closure, a `verify` scoped to the primary stopped being a
narrow check and became a false one: it reported `PASS` for work that had left
the rest of the chain stale.

This was observed exactly once, in a live run. An agent asked to propagate a
new task state amended the decision in the meeting note two hops upstream —
correctly — then ran `verify`, got `PASS`, and reported success in good faith.
`hickory test` failed on six documents whose `.md` and tangled ticket outputs
had never been regenerated. The agent was not wrong to trust its tool; the
tool was wrong.

Each upstream document is executed in its own directory, because that is where
its committed outputs live. Weaving an upstream document into the primary's
directory would create files nobody checks while leaving the real ones stale —
the same failure in a new location.

---

Last LLM verification:
- Date: 2026-08-07
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-agent/src/tools/mod.rs` — `verify()` calls
  `verify_upstream()` before executing the primary; that walks
  `EditSession::upstream`, runs `run_pipeline_live` per document with
  `working_dir` set to the document's own parent, writes its output files
  there, and collects failed expectations. The failures are merged into the
  primary's, so any upstream problem turns the whole `verify` into a `FAIL`;
  the success message appends "also re-wove N upstream document(s)".
- Test coverage: `crates/hickory-agent/tests/edits_the_chain.rs` —
  `verify_reweaves_upstream_outputs_too` edits a decision two hops up, runs
  `verify`, and asserts both the report wording and that `decisions.md` and
  `domain.md` on disk carry the new decision. Confirmed live against
  `docs/todo-app/`: `PASS — … ; also re-wove 3 upstream document(s)`.
