# `hick test` Never Spends The Reader's Tokens

Given a document containing a `<hick:agent>` cell, when `hick test` runs it,
then **no model is called**, no matter what credentials the machine holds. The
cell is verified against its recording, or reported **unverifiable** (exit `2`)
with a message naming the cell, its prompt, and how to establish a baseline.

Only `hick run` may buy a model turn.

## Why

`agent-cells.md` names the hazard directly: an agent cell makes running an
untrusted document **spend the reader's tokens**, a new hazard class on top of
"runs commands as your user". A verifier that spends money is a verifier nobody
can point at a document they did not write — which would make `hick test` on
a cloned repository an unsafe act, and `hick test docs/` in CI a billed one.

There is also nothing to gain. An agent cell is nondeterministic; re-running it
and comparing the answer would fail on wording alone. Checking it against a
recording is the only verification that means anything, which is exactly what
`freeze` already provides for every other cell whose output legitimately moves.

## What the reader sees

A cell with no baseline is reported, not fatal, and the rest of the document
still weaves — the blast radius is the cell. The report says:

- which cell (line number; an agent cell has **no** container to name),
- its prompt,
- that this run had no agent runner and why that is expected,
- how to record a baseline (`hick run --cache …` on a machine with a key,
  or plain `hick run` when the cell declares `freeze="true"`),
- and, when the cell declares no `model=`, that it should, because an agent
  recording is keyed by prompt *and* model and there is no runner here to ask
  which model would have run.

A machine with **no** provider key takes the same path in `hick run`: the
cell is unverifiable rather than an error, per `third-party-integration-mocking`
— a missing credential degrades gracefully instead of blocking unrelated work.

---

Last LLM verification:
- Date: 2026-08-10
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-cli/src/lib.rs` — `run_doc_cached` builds the agent
  runner only under `(mode == RunMode::Execute)`, so `RunMode::Verify` (what
  `hick test` uses) and `RunMode::Weave` always pass `agent_runner: None`.
  `LlmAgentRunner::from_env` (`crates/hickory-cli/src/agent_cell_runner.rs`)
  returns `None` rather than erroring when no provider key is present.
  `crates/hick-literate/src/lib.rs` records `NoBaseline::AgentWithoutRunner`
  when `collect_unverifiable` is set and fails with an actionable error
  otherwise; `unverifiable_message` in `crates/hickory-cli/src/lib.rs` renders
  the report, branching on whether the cell declared a model.
- Test coverage: `crates/hickory-cli/tests/agent_cell_vertex.rs` —
  `test_reports_an_agent_cell_with_no_baseline_as_unverifiable` (drives the
  real `run_doc` in `RunMode::Verify`, asserts the containerless `CellId`,
  `CheckOutcome::Unverifiable`, exit `2`, and the message's content) and
  `a_recorded_agent_cell_verifies_without_a_model`.
  `crates/hick-literate/tests/agent_cells.rs` —
  `an_agent_cell_without_a_runner_is_unverifiable_not_fatal`.
- Caveat requiring LLM review: `hick run` still calls a model for an agent
  cell in an untrusted document, which is the hazard `agent-cells.md` says
  capability enforcement should land before, not after. Nothing here mitigates
  that; this guarantee only removes the *verification* path as an attack
  surface. The tests are hermetic by construction — `RunMode::Verify` never
  builds a runner — so they pass identically with or without a key in the
  environment.
