# The Editable Set Is the Pipeline Closure

Given an agent session opened on a document that declares `hick:upstream`
edges, when the agent reads that document, then the result names every
document reachable through those edges — and `read_doc` and `edit_doc` accept
a `doc` argument selecting any of them, by path or by bare file name.

The chain exists so that a decision is written down exactly once. An agent
scoped to a single document cannot honour that. Told "this requirement is
wrong" when the fact actually lives in a meeting note two hops up, the only
edit available to it is a local one — so it restates the upstream fact in a
second place and produces the precise contradiction the chain was built to
prevent. Worse, it looks like success: the document it was pointed at now
says the right thing.

So the editable set is the pipeline closure, and the documents declare it
themselves. There is no flag to remember and no path to pass; the same
`hick:upstream` edges that `hickory test` follows are the ones the agent may
edit. The system prompt names the rule directly — fix a disagreement where it
is recorded, never restate an upstream fragment inline.

Closure loading is breadth-first with a visited set, so a diamond loads once
and a cycle terminates. Unreadable or unparseable edges are skipped rather
than failing the session: an agent should not be blocked from opening a
document because a different document in the chain is broken — that is what
`hickory test` reports, and fixing it may be exactly why the agent was
called.

An upstream edit parses, then writes, then re-weaves the primary. If the
re-weave fails the write is rolled back and the result says nothing changed,
because a chain edited to a state the primary can no longer weave is worse
than no edit at all.

---

Last LLM verification:
- Date: 2026-08-07
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-agent/src/tools/mod.rs` — `upstream_closure()`
  walks `hick:upstream` breadth-first from the primary; `EditSession::upstream`
  holds path → source; `resolve_target()` matches full path, suffix, or file
  name; `read_doc` appends the reachable list; `edit_doc` routes a non-primary
  `doc` argument to `edit_upstream()`, which rolls the write back if the
  primary's re-weave fails. `crates/hickory-agent/src/protocol.rs` documents
  both arguments and adds doctrine step 5.
- Test coverage: `crates/hickory-agent/tests/edits_the_chain.rs` —
  `the_agent_edits_a_decision_two_hops_upstream` asserts the primary's read
  advertises the chain and that a two-hop read by bare file name returns the
  decision; `an_upstream_edit_rewrites_the_source_and_reaches_the_primary`
  asserts the upstream source changed on disk AND that the fact was not
  restated downstream. Driven live against `docs/todo-app/` with a decision
  that had to be recorded two hops up and propagated to code.
