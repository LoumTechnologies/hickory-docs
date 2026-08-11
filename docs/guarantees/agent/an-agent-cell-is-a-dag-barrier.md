# An Agent Cell Is A DAG Vertex, And Its Edges Are A Barrier

Given a document containing `<hick:agent>`, when the pipeline builds the
information-flow DAG, then the cell is a **vertex** — scheduled by the same
topological loop as `<hick:exec>`, covered by the same freeze, recording,
`never_run`, and `hick:expect` machinery — and its edges are a **barrier**:
every cell declared before it precedes it, and every cell declared after it
follows it.

Placement was settled by the spike in
`docs/specs/freeform/agent-placement-spike.md` (exec, not flow). This guarantee
is the part that spike deferred: the cell running *from inside* the topological
loop rather than before it.

## Why a barrier, and not something cleverer

Every other edge in this graph is derived from a **declared** read or write: a
`mount=`, a `<hick:copy>` id, a container name, a `<hick:fork>`. An agent cell
declares none of those and cannot, because its read-set and write-set are
whatever the model decided to look at and edit — knowable only *after* it runs.

The sound closure over an unknown read/write set is "reads everything already
produced, writes everything not yet consumed", which is exactly a barrier in
document order. Three consequences follow, and they are the reason this is the
right conservative choice rather than merely the safe one:

- **An `<hick:exec>` written after an agent cell observes its edits in the same
  pass** — criterion 3(b), the second thing that disqualified flow placement.
  It works because the barrier guarantees that exec has not run yet.
- **The graph stays acyclic for free.** Every barrier edge points from a lower
  document index to a higher one.
- **Parallelism is only lost where it was never sound.** Cells that do not
  cross an agent cell keep their existing edges and their existing
  concurrency.

## Identity: containerless

A `<hick:exec>` cell names the container it runs in; an agent cell has none.
Its `CellId` therefore has `container: None`, which is what `CellId` being a
struct rather than a `(container, line)` tuple exists for. `hick:expect` keys on
the same `CellId`, generalizing the key exactly as `never_run` was generalized.

`ExecInfo` still carries `container: String` and `command: String`, deliberately
**not** restructured: an agent cell gets a reserved synthetic container name
(`_agent_<index>`, the trick `<hick:script>` already uses) and its prompt as the
command, so every consumer keyed on those — the recording directory, the
executor's transcript map, the live exec hook — keeps working unchanged. The
asymmetry is intentional and worth stating plainly: **identity is
containerless, storage is named.** Restructure `ExecInfo` when something
actually forces it, not before.

## Not guaranteed

- **A cell the agent inserts *before* its own cell does not run in that pass.**
  Re-preparation resumes after the agent's barrier, which is exactly the set of
  cells that have not run yet; anything the agent writes above itself is
  behind the cursor. It runs on the next `hickory run`.
- **The agent cell contributes no `SourceOrigin::Agent` provenance yet.** Its
  bytes reach lineage as ordinary `Literal` spans, because `edit_doc` puts them
  in the document before the graph is built — which is what the spike measured
  and is already correct for `hickory lineage` + `git blame`. Naming the
  session on those spans is separate, additive work.
- **Capabilities are not re-minted mid-pass.** A `<hick:container>` the agent
  declares is picked up (capabilities and image), but tokens were minted during
  preparation. An agent that needs a genuinely new capability gets it on the
  next run.

---

Last LLM verification:
- Date: 2026-08-10
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hick-exec/src/dag.rs` — `build_dag`'s pass-1 `"agent"` arm
  calls `extract_agent_info`, which assigns `agent_container_name(index)` and
  the prompt as `command`, and fills the single additive `ExecInfo.agent:
  Option<AgentCell>` field. Pass 2 emits `DependencyReason::AgentBarrier` edges
  in both directions for every agent vertex. `crates/hick-literate/src/lib.rs`
  schedules the vertex inside `run_pipeline_live`'s topological `while` loop
  and keys it with `CellId::containerless(source_line)` (see `cell_id_of`).
  `crates/hick-literate/src/expect.rs` — `collect_from_nodes` now matches both
  `exec` and `agent` and keys `ExpectSpec` by `CellId`.
- Test coverage: `crates/hick-exec/src/dag.rs` —
  `agent_cell_is_a_dag_vertex_with_a_reserved_container`,
  `an_agent_cell_is_a_barrier_in_both_directions`,
  `two_agent_cells_are_ordered_by_barriers_not_container_state`,
  `an_agent_cell_without_a_prompt_is_rejected`,
  `a_non_numeric_max_turns_is_rejected`.
  `crates/hick-literate/tests/agent_cells.rs` —
  `an_agent_vertex_runs_from_inside_the_topological_loop`,
  `an_exec_the_agent_writes_runs_in_the_same_pass`,
  `an_agent_cell_without_a_runner_is_unverifiable_not_fatal`,
  `an_expectation_on_an_agent_cell_is_evaluated`.
  `crates/hickory-cli/tests/agent_cell_vertex.rs` —
  `the_shipped_runner_settles_the_vertex_and_its_edit_lands_in_the_document`.
- Caveat requiring LLM review: the three "not guaranteed" items above are
  documented limits, not mechanical ones. Nothing stops an agent from
  inserting a cell above itself; the run simply will not execute it that pass,
  and no diagnostic says so.
