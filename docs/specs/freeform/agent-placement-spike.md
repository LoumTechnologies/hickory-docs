# Agent placement spike: flow vs exec

Status: **decided — exec** (2026-08-09). Closes the one open question in
`docs/specs/freeform/agent-cells.md`; tracked as GitHub issue #4.

Spike code: `crates/hickory-cli/tests/spike_agent_placement.rs` (11 tests, all
passing, **zero API calls** — both arms are driven by `ScriptedLlmClient`).
It is throwaway and is not wired into the `hickory` binary. Delete it when the
losing placement is deleted.

## The question

Where does a `hick:agent` node live?

- **flow** — a node converged during weave. The loop lives inside
  `get_stream`, `Context`'s TypeMap supplies the `LlmClient`/`Executor`, and
  `converge` settles on stream end.
- **exec** — a DAG vertex that runs before weave, whose edits trigger
  re-evaluation.

## Recommendation

**Exec.** It wins criterion 1 outright — which the issue predicted would be
decisive — and criterion 3 is disqualifying for flow on its own. Nothing
separates them on cost.

Delete the flow placement. Do not ship a `mode=` attribute.

## What each arm actually is

Neither placement exists in the product, so each arm realizes the placement's
*definition* against the real pipeline rather than mocking it:

- The **flow arm** is a real `hick_flow::Node` (`AgentFlowNode`) converged
  through `converge_with_provenance`. Its loop is inside `get_stream`, it
  pulls `AgentDeps { LlmClient, Executor, doc_path }` out of
  `Context::get_extension`, it emits **once** at settle via
  `futures::stream::once`, and on failure it ends the stream carrying an error
  value rather than panicking — exactly the Node semantics section of
  `agent-cells.md`.
- The **exec arm** runs the same loop as a scheduled vertex before the output
  phase, with the cell present in the document as a DAG-visible, freeze-able,
  cache-keyed cell. That is what "a DAG vertex" means to every downstream
  consumer: `build_dag`, the topological loop in
  `hick-literate/src/lib.rs`, `never_run`, and `cache::exec_cache_key` all
  identify a cell by `(container, source_line)`.

Both arms run the **same** agent loop (`run_scripted_agent`) against the same
`ScriptedLlmClient` script — one `edit_doc` turn, then `done` — so placement
is the only variable.

## The structural fact everything else follows from

`run_pipeline_live` has exactly two phases, in this order
(`crates/hick-literate/src/lib.rs`):

1. `prepare_pipeline` → `build_dag` → **`for exec_id in
   flow_dag.topological_order()`** (line ~776). Freeze, the recording cache,
   `hick:expect`, `never_run`, and transcript injection all live here.
2. **`process_pipeline_outputs`** (line ~404). It *assembles* the whole node
   graph and calls `.close()` on every `InsertionPoint`, runs
   `weave::process_weave_output`, runs the `max_rounds` re-evaluation loop —
   and only then converges.

So the graph is fully assembled and closed **before** any node's `get_stream`
runs, and the `max_rounds` re-evaluation loop runs before convergence too.
A flow-placed agent therefore executes strictly *after* the last point at
which the pipeline can react to anything.

## Evidence, criterion by criterion

### 1. `check` parity — **fails; decisive** (`c1_*`)

The same never-run agent cell gets **different verdicts**:

| Placement | `never_run` | `CheckOutcome` | exit |
|---|---|---|---|
| flow | empty | `Verified` | 0 |
| exec | 1 entry | `Unverifiable` | 2 |

A flow-placed agent cell is not a DAG cell, so it contributes no `never_run`
entry and `check` has nothing to call unverifiable. `check` therefore reports
a document **verified** whose agent cell has never run — precisely the failure
`415d430` was written to prevent ("we have verification" quietly becoming "we
have verification for the parts that ran").

Under exec placement the freeze machinery from `cea27b6` covers the cell for
free. `c1_exec_placement_verifies_against_a_recording` records the cell with
caching on, flips it to `freeze="true"`, and gets `Verified` — the replay
direction works with no new mechanism. There is no flow equivalent, because
the recording path is inside the topological loop a flow node never enters.

This alone ends it, as the issue predicted.

### 2. Re-entrancy — **flow does not corrupt state, but cannot propagate** (`c2_*`)

`EditSession::reweave` calls `hick_literate::run_pipeline_weave`. A
flow-placed agent's `edit_doc` therefore re-enters the pipeline from inside
the converge that is producing that document's output.

Measured result: **no deadlock and no corruption** — the nested weave runs to
completion, the edit lands on disk, the outer converge settles. But the outer
converge returns the *old* document's output
(`c2_flow_agent_edit_is_invisible_to_the_run_that_produced_it` asserts the
converged text does not contain the agent's bytes), because the graph closed
before converge began. A second whole-pipeline pass is required, and nothing
in the pipeline schedules one — `max_rounds` only re-parses generated `.hick`
files, and it has already finished by the time the agent runs.

Under exec placement the edit lands before assembly, so one pass carries it
(`c2_exec_agent_edit_is_visible_in_the_same_pass`).

Special cases needed: flow needs an outer re-run loop that does not exist
today plus a fixed point for it; exec needs `prepare_pipeline` re-run after an
agent vertex edits the source. Exec's is one loop in one place; flow's is a
new control structure above `run_pipeline_live`.

### 3. Ordering, both directions — **flow supports only one; disqualifying** (`c3*`)

- **(a) agent consumes an exec's output** — works under both. Exec
  transcripts exist before the output phase, so a node converged during weave
  can read them, and a DAG successor can too.
- **(b) an exec consumes the agent's edits** — works **only** under exec.
  `c3b_only_exec_placement_lets_an_exec_consume_agent_edits` runs the same
  document twice: in the flow arm the consuming exec prints `MISSING` because
  every exec in the document has already finished by the time the agent runs,
  and its transcript for that pass can never change; in the exec arm the same
  exec prints `hello from the agent`.

The issue declared "supporting only one direction is disqualifying".

### 4. Termination — **flow's blast radius is the document** (`c4_*`)

`c4_flow_a_non_completing_node_hangs_the_whole_document`: a node returning
`stream::pending()` inside an `InsertionPoint` alongside two `StringNode`s
leaves `converge_with_provenance` blocked past a timeout. The literal prose
either side is lost too — converge keeps the last batch and there is no last
batch. There is no per-cell timeout anywhere in the converge path.

`c4_flow_turn_budget_ends_the_stream_instead_of_hanging`: the only thing that
saves the flow arm is `max_turns` *inside* the node, and only because the
spike's loop deliberately ends the stream carrying an error value. That
confirms the design's rule ("on error the stream must end") is load-bearing
rather than stylistic — and that it is the sole line of defence.

`c4_exec_a_cell_without_a_baseline_is_reported_not_fatal`: under exec, a cell
with no baseline is *named*, with a reason and a remedy, and the rest of the
document still weaves. Blast radius is the cell.

Exec is not immune — a hanging vertex hangs the topological loop — but it
inherits `ScriptLimits` (timeout + output cap) and a per-cell error path that
still yields a transcript entry, neither of which exists in converge.

### 5. Lineage completeness — **only exec resolves** (`c5_*`)

`SourceOrigin` has no `Agent` variant today. More importantly, adding one
would not fix flow: a flow-emitted byte has no byte-precise source span, so
`hickory_lineage::from_provenance_map` classifies it non-editable and
`map_edits` would reject any edit crossing it (`LineageError::SyntheticOverlap`).
The spike asserts `origin.source().is_none()` for the agent's bytes in the
flow arm.

Under exec the agent's bytes reach lineage as ordinary `Literal` spans,
because `edit_doc` puts them in the document *before* the graph is built. The
spike asserts `output_lineage(&run, "greeting.txt")` yields at least one span
with an editable source. So `hickory lineage` + `git blame` compose exactly as
the "No `author` field" section of `agent-cells.md` requires — and the
no-write-primitive constraint is what makes this true, not the placement
choice per se. Placement only decides whether the run observes the edit.

`SourceOrigin::Agent { session, turn }` is still worth adding under exec, to
name the session on a span that already has a document location. It is
additive and does not change this result.

### 6. Cost — **identical; separates nothing** (`c6_*`)

Same loop, same script, same turns:

| | turns | input | cache write | cache read | output | USD |
|---|---|---|---|---|---|---|
| flow | 2 | 1500 | 400 | 1200 | 110 | 0.00801 |
| exec | 2 | 1500 | 400 | 1200 | 110 | 0.00801 |

Recorded through the harness's own `RunRecord` + `generate_report` so the
four-way split is measured with the instrument the issue named. Placement does
not move a token, and it was never plausible that it would; the criterion is
recorded as *not* separating them.

## A note on the harness

`ArmSpec` has no placement axis, and adding one would mean shipping a
placement knob in production code — the exact thing the issue's deletion
condition forbids. So the two arms are built directly in the spike test and
only the *measurement* (`RunRecord`, `generate_report`) is taken from
`hickory-agent::harness`. That is a small piece of evidence in its own right:
placement is not an experiment dimension, it is a decision.

## Scorecard

| Criterion | flow | exec |
|---|---|---|
| 1. `check` parity | verified-when-never-run (wrong) | unverifiable / verified-on-replay |
| 2. Re-entrancy | safe but cannot propagate; needs a new outer loop | one pass; re-prepare |
| 3. Ordering (a) agent←exec | works | works |
| 3. Ordering (b) exec←agent | **impossible in-pass** | works |
| 4. Termination | hangs the whole document; no per-cell bound | cell-scoped, reported |
| 5. Lineage | synthetic, non-editable | `Literal`, editable |
| 6. Cost | identical | identical |

Not a close call, and not a "no clear winner" result: criteria 1 and 3 each
end it independently, and 2, 4, and 5 point the same way.

## What exec placement now owes

Recorded so the follow-up work is not discovered later:

1. `ExecInfo.container` is a plain `String` and `command: String` — neither
   describes an agent cell. `CellId.container` is already `Option<String>`
   (`415d430`), so the pattern is set; `ExecInfo` has to follow, or the agent
   cell gets a reserved synthetic container name and every consumer keyed on
   `(container, source_line)` keeps working unchanged. Prefer the latter until
   something forces the former.
2. `build_dag`'s pass-1 match (`crates/hick-exec/src/dag.rs`) needs an
   `"agent"` arm, and its edges must come from something other than
   copy/paste and volumes — an agent cell's read-set and write-set are known
   only after it runs.
3. `cache::exec_cache_key(image, caps, command, secrets)` has no input that
   describes an agent turn. A recording key for an agent cell has to include
   the prompt and the model, or freeze silently verifies the wrong thing.
4. Re-preparing the pipeline after an agent vertex edits the source needs a
   declared fixed point and a bound, or a document with two agent cells can
   re-prepare forever.
5. `expect::collect_expectations` keys on `(container, exec_line)`; a
   containerless cell needs a different key if `hick:expect` is ever allowed
   on an agent cell.

## Threat to validity

The exec arm models "runs before the output phase" by running the loop before
`run_pipeline_weave` / `run_doc` rather than from inside the topological loop,
because putting it inside the loop requires the `build_dag` and `ExecInfo`
changes listed above — which is work the spike deliberately does not do. The
ordering property being measured (exec phase strictly precedes output phase)
is a property of `run_pipeline_live`'s control flow, not of where the call
sits, so the substitution does not affect criteria 2, 3, or 5. It does mean
criterion 4's exec half is measured on a *frozen exec cell* standing in for an
agent cell rather than on a real agent vertex; the claim it supports — one
unverifiable cell is reported rather than fatal, and the rest of the document
still weaves — is a property of the loop the agent vertex would live in.
