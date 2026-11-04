# Reactive algorithm benchmark v4

For the maintainer running the next prediction-gate evaluation and writing up its limits.

The completed smoke pilot (2026-11-03 20:08:29) was valid: baseline and gate each
passed 3/3 tasks on their first proposals, with no prediction mismatches or blocked
saves. It established that the harness works and exposed a ceiling on those tasks.
The user then explicitly requested more complex LiveLinq and DynamicCombineLatest
data. This new task population is versioned separately; it must not be pooled with
the smoke pilot or described as the original preregistered main experiment.
The earlier invalid protocol-2 run also remains retained as an infrastructure failure.

## What the new suite exercises

These are self-contained Python semantic extractions, not native C#/Rx.NET or
Rust/Tokio integration tests. Starting implementations contain deliberately seeded
faults. They are benchmark repair exercises, not claims that the upstream projects
have those exact defects. The source references identify the inspected mechanisms;
the task text is the complete benchmark contract.

| Task | Interactions checked |
|---|---|
| DCL readiness and retained values | Pending sources, completed cached values, late data after removal/completion, numeric source order |
| DCL queued quiescence | FIFO draining, one emission per poll, queued additions behind completion, terminal state, shutdown before queued data |
| LiveLinq list flattening | Shared inner source aliases, callback creation order versus outer order, per-callback offsets, range removal, detachment and disposal |
| LiveLinq persistent grouping | Same-key value changes, duplicate source IDs, stable group identities through empty/repopulate cycles, stale updates |
| LiveLinq distinct ownership | Multiple owners of one inner source, overlapping values, zero/positive reference-count transitions, reattachment and frozen disposal |
| LiveLinq index-sensitive filtering | Range edits shifting every downstream index, predicate toggles, projection updates, canonical remove/add order and unsubscribed indices |

Each task has two public traces and twelve disjoint held-out traces: 12 public and
72 held-out cases across six task variants. Held-out traces have 44–79 actions,
from fixed seeds 1100–1111 (the second DCL variant offsets those seeds by 100).
Two DCL variants share an algorithm, so six variants are not six independent
algorithm families. Repeated trial sessions are also not new task types.

Public prediction questions require the exact complete output for both public
traces, including delta logs, state snapshots and final subscriptions where the
API returns them. The gate never receives held-out cases. The model receives only
the task's starting kernel, complete repair requirements, public inputs, restricted
Python interface and provenance references. Evaluator solutions, generators,
oracles, hidden inputs and scores stay outside its conversation and behind the
existing filesystem quarantine.

The LiveLinq reference is pinned to
[`a50ba1976b78b0edf635a96074d1895812141564`](https://github.com/nloum/LiveLinq/tree/a50ba1976b78b0edf635a96074d1895812141564).
Inspected boundaries include
[`SelectMany` offsets and disposal](https://github.com/nloum/LiveLinq/blob/a50ba1976b78b0edf635a96074d1895812141564/LiveLinq/Extensions.List.SelectMany.cs),
[`MovingRanges` invalidation](https://github.com/nloum/LiveLinq/blob/a50ba1976b78b0edf635a96074d1895812141564/LiveLinq/List/MovingRanges.cs),
[retained empty groups](https://github.com/nloum/LiveLinq/blob/a50ba1976b78b0edf635a96074d1895812141564/LiveLinq.Tests/Extensions.Set.GroupBy.Tests.cs),
[set flattening](https://github.com/nloum/LiveLinq/blob/a50ba1976b78b0edf635a96074d1895812141564/LiveLinq/Extensions.Set.SelectMany.cs), and
[index-sensitive predicates](https://github.com/nloum/LiveLinq/blob/a50ba1976b78b0edf635a96074d1895812141564/LiveLinq/Extensions.List.Where.cs).
The local reference is
[`DynamicCombineLatest`](../../crates/hick-flow/src/combine_latest.rs)
at repository revision `6662e6566f9df595747248382317291a92fb0400`.
ComposableCollections was an older home for LiveLinq and is not a second source.

## Keep reference answers independent

Correct kernels are compared against evaluator-only reference reducers. Collection
oracles recompute views from source state rather than calling the candidate's
incremental helpers; DCL uses a different record representation and mailbox cursor.
Hand-written witnesses additionally fix alias callback order, empty-group identity,
ownership multiplicity, queued additions, shutdown priority and index-shift deltas.
Agreement is evidence within this finite suite, not proof that either oracle is
universally correct. No model outcomes were used to choose the held-out seeds.

A runtime exception in one case now produces an explicit failed-case marker while
preserving other case outcomes. Timeouts or whole-module admission failures can
still fail an entire case batch; the receipt distinguishes them. JSON comparison
checks types as well as values, so `true` cannot silently satisfy an expected `1`.
The AST policy admits ordinary list/set mutation and lambdas, while retaining the
import/reflection/private-name restrictions. The exact builtin/method interface
is included in both conditions' task text. Observation output is bounded at 4 MiB.

## Freeze before live calls

The primary outcome and success rule are unchanged: all scheduled sessions count;
final accepted code must pass every held-out case; compare paired gate minus
baseline success rate; require at least +10 percentage points and a paired bootstrap
95% lower bound above zero for the declared success criterion. Anything else is
inconclusive for benefit, with failures and any deterioration reported explicitly.
A degenerate bootstrap interval from all ties does not demonstrate equivalence.

Both conditions use the same model and low reasoning effort, six response attempts,
360-second session budget, 60-second transport timeout, 96,000-byte cumulative
replayed-input budget, 32,000-byte accepted answer limit and candidate execution
limits. API mode additionally requests at most 8,192 output tokens per call; the
subscription CLI cannot enforce that generation-token cap. Its answer-size limit
is checked after generation and is not a credit cap. These larger limits apply
because the task population changed, before any reactive-suite model comparison.
Do not mix these runs with the old 180-second smoke profile.

Pilot: one repeat per task, six pairs / twelve sessions. Main: four repeats per
task, 24 pairs / 48 sessions. Validate transport, accounting, confinement and gate
execution in the pilot before starting main; do not select cases or alter success
criteria based on which condition wins. All six frozen variants remain in main,
including easy, failed or nondiscriminating ones. Any repair to the benchmark or
runner after new outcomes requires another version and rerunning both arms.

At standard Sol credit rates already documented in the runbook, a planning
scenario of 12,000 uncached input and 3,000 output tokens costs about 1.35 credits
per turn. That is about 16.2 credits for a one-turn-per-session pilot or 64.8 credits
for a one-turn main; six turns per session would be about 97.2 and 388.8 credits.
These are scenarios, not enforced spending caps or verified debits. CLI overhead,
reasoning, caching, retries and actual output sizes change usage. No model calls
are made by the offline commands below.

## Reproduce

Build the unchanged production output adapter if needed, then run only the focused
integrity checks:

```sh
cargo build --locked -p hick-literate --example prediction_bench
python3 -m unittest discover -s experiments/prediction-gate -p 'test_*.py' -v
```

Exercise the whole transport-independent harness without paid calls:

```sh
python3 experiments/prediction-gate/bench.py --simulated --suite reactive \
  --out target/prediction-gate-reactive-simulated
```

The model comparison runs from a normal Terminal using the existing ChatGPT sign-in:

```sh
python3 experiments/prediction-gate/bench.py --live --backend codex --suite reactive \
  --model gpt-6.1-sol --reasoning low --repeats 1 \
  --out "target/prediction-gate-reactive-pilot-$(date +%Y%m%d-%H%M%S)"
```

After validating that pilot, the main command changes `--repeats 1` to
`--repeats 4` and uses a fresh `reactive-main` output directory. Do not use the old
smoke command without `--suite reactive`: it intentionally retains the old tasks.
Preserve the manifest, actual parser warnings, every proposal/prediction/gate result,
held-out score and accepted final patch. The manifest hashes the selected fixture
module, runner, Codex transport and production gate binary and records all limits.
Keep output directories evaluator-only throughout all runs.

## What this can support in the blog

Report the old smoke pilot as an honest engineering checkpoint, then explain the
user-requested new population and larger shared budgets before presenting v4 data.
Show exact traces where a removed alias still affects an offset, an empty group
retains identity, or queued input changes a quiescence decision. Use actual archived
model proposals for claims about predictions and repairs; reference-kernel examples
are not model evidence. Report all tasks, first/final correctness, discordant pairs,
uncertainty, errors, retries and token/credit costs without pooling versions.

A pass demonstrates behavioral performance on deterministic extracted contracts.
It does not establish a mental model, native Rx.NET/Tokio correctness, scheduler
fairness, cancellation wake-up behavior, memory safety, thread races or realistic
large-project coding productivity. Snapshot-recomputing solutions can pass when
behavior is correct; asymptotic efficiency is not an outcome of this study.
