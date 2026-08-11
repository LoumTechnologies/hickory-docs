# Re-Preparation After An Agent Edit Terminates, Under A Declared Bound

Given a document whose `<hick:agent>` cell edits that document's own source,
when the pipeline re-prepares the document so the rest of the pass runs against
what the agent wrote, then re-preparation reaches a **declared fixed point**
under a **declared bound**, and a document with no fixed point **fails** rather
than looping.

## The fixed point

> A pass is finished when it reaches the end of the topological order with no
> agent cell having edited the source.

Concretely: when an agent vertex reports `edited_source`, the pipeline re-reads
the document from disk, re-parses it (includes, vars, conditionals), rebuilds
the DAG, re-collects `<hick:container>` declarations and `<hick:expect>`
specs, re-seeds input volumes that no cell has written to yet, and **resumes
after that cell's barrier**. Resuming after the barrier is sound precisely
because the barrier is what guarantees every remaining cell has not run: no
cell runs twice, and no already-run cell is re-executed.

## The bound

> The number of agent cells the document declared when it was first parsed.

This is not an arbitrary ceiling; it is what the fixed point implies. Every
re-preparation is caused by a distinct agent cell running, and a cell runs at
most once per pass, so a document with *n* agent cells re-prepares at most *n*
times. Exceeding it means one thing: **an agent authored another agent cell**,
and the document has no fixed point.

Exceeding the bound is a failure with a message that says so, not a truncated
run reporting partial results. This is the same class of invariant as
`max_turns` (`agent-cells.md`, "max_turns is a graph invariant"): a run that
cannot reach a fixed point fails, because a downstream consumer silently
accepting a partial result is the failure mode the invariant exists to prevent.

`PipelineConfig::max_agent_reprepares` overrides the bound for a caller that
genuinely intends the extra round. `0` (the default) means "derive it".

## Identity across the agent's own edits

An agent's first act is usually to insert text, which moves every source line
below it — including its own. Resuming therefore cannot key on a line number.
The cell is identified by its `id=` when declared, otherwise by its ordinal
among the document's agent cells. A cell that deletes or renames itself makes
the resume point unrecoverable and fails with a message saying to give it an
`id=`.

## Not guaranteed

- **Cells before the barrier are not re-run** after a re-preparation, even if
  the agent edited them. Their transcripts stand. Re-running them would mean
  repeating side effects that already happened, which is a worse failure than
  a stale cell above the agent.
- **No fixed point is sought across documents.** One document's agent editing
  *another* document is not detected, and that other document is not
  re-prepared if it has already been processed.

---

Last LLM verification:
- Date: 2026-08-10
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hick-literate/src/lib.rs` — `run_pipeline_live` iterates
  documents by index and each document's cells with an explicit `cursor` over
  `flow_dag.topological_order()`. The agent arm computes `reprepare_budget`
  from `config.max_agent_reprepares` or the initial agent-cell count, increments
  `reprepares` on every `edited_source`, and `anyhow::bail!`s when it passes the
  budget. `reprepare_document` re-reads and re-parses from disk;
  `collect_container_defs`, `collect_expect_specs`, and `seed_input_volumes`
  (with the already-written volumes skipped) refresh the derived state; the
  cursor is set to one past the position of the cell whose `AgentCell::key()`
  matches, and a missing cell fails with the "no longer in it" error.
- Test coverage: `crates/hick-literate/tests/agent_cells.rs` —
  `re_preparation_terminates_at_its_declared_bound` (an agent that appends
  another agent cell every time fails at the bound, and the message names the
  fixed point and `max-turns`), and `an_exec_the_agent_writes_runs_in_the_same_pass`
  (the fixed point is actually reached, and the agent's cell ran).
  `crates/hickory-cli/tests/agent_cell_vertex.rs` —
  `the_shipped_runner_settles_the_vertex_and_its_edit_lands_in_the_document`
  covers the same loop under the shipped runner.
- Caveat requiring LLM review: the two "not guaranteed" items are documented
  limits with no diagnostic. In particular, a cell the agent inserts *above*
  its own cell is silently not run in that pass; only the next `hickory run`
  picks it up.
