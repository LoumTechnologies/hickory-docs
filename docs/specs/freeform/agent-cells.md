# Agent cells (`hick:agent`)

Status: **design settled** (2026-08-09). The last open question — node
placement — was decided by a spike rather than by argument; see
"Placement: settled" and `docs/specs/freeform/agent-placement-spike.md`.

An agent cell puts a reasoning step *inside* a document, alongside `hick:exec`,
instead of leaving the agent as a CLI verb that acts on documents from outside.
A pipeline can then contain the reasoning that produced it, and that reasoning
enters the provenance graph as a first-class origin.

```xml
<hick:agent id="impl-tokenizer" max-turns="20" model="claude-sonnet-5">
  <hick:prompt>Implement the tokenizer described above; keep the examples passing.</hick:prompt>
</hick:agent>
```

`id=` identifies the cell across the agent's own edits — line numbers move the
moment it inserts anything. `model=` is optional, and is what lets the cell be
verified from its recording on a machine with no credentials. `freeze=` works
exactly as it does on `<hick:exec>`.

## The central constraint: no write primitive

**`edit_doc` and `edit_output` are the agent's only way to write.** There is no
raw file write, and no escape hatch.

Every byte the agent produces therefore passes through `hickory-lineage`
(`map_edits` → `apply_source_edits`), which makes several properties structural
rather than disciplinary:

- **Provenance is complete by construction.** There is no unprovenanced write
  path to forget to instrument.
- **The agent cannot create drift.** It cannot write an output except *through*
  the source that generates it, so `check` on agent work is close to a
  tautology.
- **Incremental re-execution is sound.** A complete write-set and a complete
  read-set — both mediated by the same tool surface — are what make a cache key
  trustworthy.
- **The capability story collapses to almost nothing.** The agent needs no
  filesystem write capability at all.

It also answers, structurally, the cause of death recorded in
`docs/market/failure-ledger.md` §1: literate programming died of the parallel
maintenance burden between a document and its derived artifacts. `edit_output`
maps edits made in the derived surface *backward* into the source, so the
document stays authoritative automatically.

The claim this produces: **the agent cannot write a byte that isn't derived
from a document you can read.**

### The obvious objection, and the answer

*An agent can simply author a `hick:exec` cell that writes anywhere.* True —
and correct. That write is then **declared in the document, visible in the
diff, and subject to capability enforcement**. The property is not "the agent
cannot do X"; it is "the agent cannot do X invisibly." Arbitrary power routed
through an artifact a human reads is the actual guarantee, and it is more
honest than pretending to prevent it.

### Consequences

- **New files are document edits** — the agent adds a `hick:file` block.
- **Build artifacts and lockfiles are `hick:exec` cells the agent authors**, not
  files it writes.
- **Formatters are pipeline transforms, not edits.** Running `cargo fmt` and
  mapping the result back is the adversarial case for lineage: a whole-file
  reformat generates edits that cross origin boundaries, and `SourceOrigin::Paste`
  already documents that dedented content carries no source span. Instead, the
  document declares that a file is rendered *through* rustfmt, and weave applies
  it — so the woven output already is the formatted output and nothing maps
  backward. `TransformNode` (`Fn(&str) -> String`) is the existing shape for
  this. The rule generalizes to linters with `--fix` and to code generators.

## Node semantics

- **Settling is stream completion.** `converge` consumes
  `BoxStream<Vec<NodeTrace>>` and keeps the last batch; `StringNode` is
  `futures::stream::once`. So "the agent is done" ends the stream, `converge`
  takes the final value, and completion propagates. `InsertionPoint` already
  models the alternative with `never_complete_if_sources_may_change`.
- **The cell emits once, at settle.** Turn-level detail goes to the session,
  not the stream. Emitting per turn would make combine-latest re-trigger
  downstream nodes on every turn — an agent piped into another agent would run
  it twenty times because its input took twenty turns. Nothing is lost, because
  `SourceOrigin::Agent { session, turn }` resolves turns from the session.
- **The loop lives inside `get_stream`.** The DAG stays acyclic; iteration is
  internal to the vertex, exactly as a `hick:exec` vertex makes many syscalls
  without the scheduler caring.
- **`Context` supplies the dependencies.** It is a type-erased extension map
  whose stated purpose is keeping `hick-flow` free of domain dependencies, so
  the agent node pulls its `LlmClient` and `Executor` from an extension and
  `hick-flow` never grows an LLM dependency.
- **`max_turns` is a graph invariant, not a cost policy.** `converge` blocks on
  stream end, so a node that fails to complete hangs the whole document, not
  just the cell. Two rules follow: on error the stream must **end** carrying an
  error value rather than propagating a panic, and exhausting `max_turns`
  without settling is a **failure**, not a partial result downstream consumers
  silently accept.

## Provenance and authorship

**Shipped** (issue #5): `SourceOrigin::Agent { session, turn, file, span }` —
an additive variant on the serde-tagged enum (`crates/hick-flow/src/node.rs`).
`converge_with_provenance` already walks a trace backward to the nearest
origin, so lineage works as soon as the variant exists and turns are injected
where exec output is injected today. `file`/`span` are optional and default to
absent: they carry the document region the agent's edit landed in when the
bytes are byte-identical to it, which is what gives `git blame` a line to
answer for. An origin serialized without them still deserializes.

The read path is complete and tested: `hickory_lineage::Origin::Agent`
(the one origin kind that never degrades to `synthetic` — losing the session id
is precisely the failure the variant exists to prevent),
`hickory_cli::agent_lineage`, and the `hickory lineage` rendering. See
`docs/guarantees/lineage/agent-lineage-degrades-without-a-session.md`.

**Issue #7 landed the vertex** (2026-08-10): `build_dag` has an `"agent"` arm,
the cell is scheduled by the topological loop, and the recording key includes
the prompt and the model. Its bytes reach lineage as ordinary `Literal` spans,
because `edit_doc` puts them in the document before the graph is built — which
is already what `hickory lineage` + `git blame` need.

**Still outstanding**: nothing *names the session* on those spans yet, so
nothing produces a `SourceOrigin::Agent`. That is now purely additive — attach
`{ session, turn }` to the spans an agent's `edit_doc` produced — rather than
blocked on the DAG. Until then, agent origins exist only as synthetically
constructed spans in tests.

**No `author` field.** Authorship composes instead: `hickory lineage` maps an
output byte to a document span, and `git blame` on that span gives the commit
author.

> `foo.rs:42` ← session `abc123` turn 7 · committed by … · reasoning not available to you

An `author` baked in at promote time would be *asserted* by whoever ran promote
— the same asserted-vs-derived distinction this project avoids everywhere else.
Git's author is anchored in a commit and can be signed. It also gets the
accountability model right: the human who ran the agent and committed the
result is accountable, not the model.

## Sessions, privacy, and promote

- **A cache may be evicted; a session must not be.** Lineage points into
  sessions, so they are durable repo state, not `.hick-cache/` content. The
  cache stores *what came out*; the session stores *how it got there*.
- **Sessions are per-author and may be private.** The shared document is what
  everyone sees; another author's reasoning may be unreadable to you.
  `hick-secrets` already vendors age encryption if sessions are to be committed
  rather than gitignored.
- **Unresolvable lineage is a designed outcome, not an error.** A byte whose
  session you cannot read still reports the session id, the turn, and the
  committing author. **Shipped** (issue #5): `resolve_reasoning` resolves
  `<project_dir>/sessions/<id>.hick` and reports *why* it could not — missing,
  unreadable, unparseable, or a turn the session does not record — rather than
  raising. `blame` does the same for authorship: no git, no repository, an
  untracked file, and an uncommitted line each resolve to a stated outcome, and
  an uncommitted line is attributed to the reader's own configured identity,
  because uncommitted work is theirs by definition.
- **`promote` is what makes a document publicly verifiable.** It strips the
  agent cell, leaving the `hick:file` / `hick:exec` cells the agent authored,
  each carrying its `SourceOrigin::Agent` reference. A promoted document
  verifies with **no session at all**; only an un-promoted document carrying a
  live agent cell needs one. Hence the discipline: **promote before you share.**

## Verification

`check` gains a third outcome. Today it collects unmet expectations and drift
into a single failure bucket, and `never_run` is consulted only in the
weave/render path.

| Outcome | Meaning |
|---|---|
| **verified** | Re-derivation matches what is committed |
| **drifted** | Something changed |
| **unverifiable** | No baseline was ever established — no recorded session, or a never-run cell |

These need distinct exit codes. Drift means someone changed something;
unverifiable means nothing was ever established. Conflating them is how "we have
verification" quietly becomes "we have verification for the parts that ran."

`freeze` is the mechanism for cells whose output legitimately moves — lockfiles,
network fetches, timestamps. It is now **per-cell**: `freeze="true"` on an exec
freezes that cell alone, `freeze="false"` keeps a cell live even under a
run-wide `--freeze`, and an omitted attribute inherits the run-wide default. So
a lockfile cell gets freeze and an integration test does not. Note the honest
cost — a frozen cell's `hick:expect` assertions pass trivially, so freeze
verifies "the document still produces what we recorded," not "the world still
agrees." See `docs/guarantees/verification/freeze-is-declared-per-cell.md`.

## Placement: settled — **exec**

**The agent node is a DAG vertex that runs before weave.** Decided 2026-08-09
by the spike in `docs/specs/freeform/agent-placement-spike.md`
(11 offline tests; the spike code was deleted with issue #7). The
flow placement — a node converged during weave — is **deleted**. Placement
does not ship as a document attribute (`mode=`), and there is no second
ordering in the vocabulary.

Both placements were built and measured against the six criteria declared
before either existed. Two of them ended it independently:

- **`check` parity failed.** The same never-run agent cell yields `Verified`
  (exit 0) under flow and `Unverifiable` (exit 2) under exec. A flow-placed
  cell is not a DAG cell, so it contributes no `never_run` entry and `check`
  reports a document verified whose agent cell has never run. Under exec the
  per-cell `freeze` machinery covers the cell for free, in both the
  no-baseline and the replay direction.
- **Ordering worked in only one direction under flow.** An agent consuming an
  exec's output works either way, but an exec consuming the agent's edits is
  impossible in a single flow-placed pass: every exec in the document has
  already finished by the time a node converges. Supporting one direction only
  was declared disqualifying.

Re-entrancy, termination, and lineage point the same way. A flow-placed
agent's `edit_doc` re-enters `run_pipeline_weave` from inside the converge —
which corrupts nothing, but the run cannot observe its own edit, because
`process_pipeline_outputs` assembles and closes the whole graph (and finishes
its `max_rounds` re-evaluation) *before* convergence begins. A node that fails
to complete blocks `converge` with no per-cell bound, taking its literal
siblings with it; an exec cell's failure is named and scoped. And bytes a flow
node emits carry no source span, so `hickory lineage` cannot resolve them and
`map_edits` would refuse to edit through them — whereas an exec-placed agent's
bytes reach lineage as ordinary `Literal` spans, because `edit_doc` puts them
in the document before the graph is built.

Cost was identical (same turns, same four-way split, same USD) and separated
nothing.

The Node semantics above still hold for the cell — settling is completion, the
cell emits once at settle, the loop lives inside the vertex, and `max_turns`
is a graph invariant — but the vertex is scheduled by `hick-exec`'s
topological loop rather than converged by `hick-flow`.

**The five follow-ups the spike recorded are done** (issue #7, 2026-08-10), and
the spike code has been deleted. What was decided:

- **The cell's edges are a barrier.** Everything declared before it precedes
  it, everything after follows it. An agent's read-set and write-set are known
  only after it runs, so the barrier is the sound closure over an unknown one —
  and it is what makes "an exec consumes the agent's edits" work in a single
  pass. `docs/guarantees/agent/an-agent-cell-is-a-dag-barrier.md`.
- **`ExecInfo` was not restructured.** The agent cell takes a reserved
  synthetic container name (`_agent_<index>`) and its prompt as the command, so
  every container-keyed consumer keeps working; `ExecInfo` gained one additive
  `agent: Option<AgentCell>` field. Cell *identity* — `never_run`,
  `hick:expect` — is containerless, which is what `CellId.container` being an
  `Option` was for.
- **The recording key is the prompt and the model.** `max-turns` is not in it.
  A cell that should replay on a machine with no credentials declares `model=`.
  `docs/guarantees/agent/an-agent-recording-is-keyed-by-prompt-and-model.md`.
- **Re-preparation has a declared fixed point and bound.** The pass finishes
  when no agent cell edited the source; the bound is the number of agent cells
  declared at first parse, and exceeding it fails rather than truncating —
  the same class of invariant as `max_turns`.
  `docs/guarantees/agent/re-preparation-terminates.md`.
- **`hickory test` gets no agent runner at all**, even where credentials exist.
  A verifier must not spend the reader's tokens, and re-running a
  nondeterministic cell would not verify anything; a cell is checked against
  its recording or reported unverifiable.
  `docs/guarantees/agent/test-never-spends-tokens.md`.

## Hazards

- An agent cell makes a document **nondeterministic by default** and makes
  running an untrusted document **spend the reader's tokens** — a new hazard
  class on top of "runs commands as your user." This is the argument for landing
  capability enforcement before the cell, not after.
- Local execution is not sandboxed; capabilities are currently *recorded, not
  imposed*, except under `hickory-executor-docker`. Be precise about that in any
  external claim: "provable" and "documented" are different words.

## Deliberately deferred

Multiple agent cells in one document (ordering, one session or several,
cross-reading). Spend caps beyond `max-turns` (`usage.rs` already computes USD).
The external policy layer (`hick-policy`), sink wiring, and taint propagation.
The MCP server and host deny rules — worth more once the no-write-primitive
property is real, at which point it holds under another harness too.
