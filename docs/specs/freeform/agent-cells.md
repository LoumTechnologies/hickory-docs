# Agent cells (`hick:agent`)

Status: **design settled, placement unresolved** (2026-08-09). One blocking
question remains — see "The open question" — and it is being decided by a spike
rather than by argument.

An agent cell puts a reasoning step *inside* a document, alongside `hick:exec`,
instead of leaving the agent as a CLI verb that acts on documents from outside.
A pipeline can then contain the reasoning that produced it, and that reasoning
enters the provenance graph as a first-class origin.

```xml
<hick:agent id="impl-tokenizer" max-turns="20">
  <hick:prompt>Implement the tokenizer described above; keep the examples passing.</hick:prompt>
</hick:agent>
```

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

`SourceOrigin::Agent { session, turn }` — a new, additive variant on a
serde-tagged enum. `converge_with_provenance` already walks a trace backward to
the nearest origin, so lineage works as soon as the variant exists and turns are
injected where exec output is injected today.

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
  committing author.
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

## The open question

**Where does the agent node live?** `hick-flow` assembles and traces;
`hick-exec` schedules; weave runs after exec. An agent whose only write channel
is `edit_doc`/`edit_output` edits the document *currently being evaluated to
produce the graph it is running in*. `EditSession` re-weaves after every edit
today, which is fine outside a converge and re-entrant inside one.

Two coherent placements:

- **flow** — a node converged during weave, per the Node semantics above.
- **exec** — a DAG vertex that runs before weave, whose edits trigger
  re-evaluation.

This is being decided by a spike with pre-declared criteria and a deletion
condition, because the answer depends on exec→weave ordering that argument
cannot settle. See the tracking issue. **Placement must not ship as a document
attribute** (`mode=`): two orderings in the vocabulary means every future
feature works twice, and `check` semantics would have to be provably identical
across modes or verification differs by mode.

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
