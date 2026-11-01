# Compare local models with historical nloum code

For maintainers running a small personal coding benchmark and inspecting readable model context.

This first curated dataset contains eight C# tasks from three `nloum` repositories: two reference-conversion repairs, an exception-factory refactor, and five method completions. Both author and committer dates precede 1 January 2021. Each record retains the commit, parent when present, original source location, raw Git blob hash, extracted method hash, prompt, broken/missing method, supplied collaborators, public example and held-out checks.

The prompts were reconstructed now from inspected behavior and commits; they are not recovered historical requests. Completion tasks replace a committed body with `NotImplementedException`; repair/refactor tasks use the historical parent method. Reference methods retain their source text with normalized newlines and are compiled in reduced fixtures. This is not a full build of the original applications. A supplied helper in the variable-name case comes from a later pre-2021 snapshot instead of restoring Humanizer; that substitution is explicit in the record.

## Qualify before asking a model

A historical reference must pass public and held-out native C# checks, and the broken/missing baseline must fail the public check. The stored receipts are bound to the exact three compiled fixture sources. Reference success is therefore an inclusion criterion, not an unbiased estimate that the human solved 100% of arbitrary tasks. The checks were authored for this study after inspecting the changes and may miss behavior outside their stated contract.

The SDK and runtime images must already be installed. Candidate code builds in a network-disabled Docker container and executes in a second restricted, read-only container that receives binaries only. There is no workspace, credential or Docker-socket mount. Time and memory are bounded, and timed-out containers are removed. This is process confinement and finite behavioral testing, not proof of semantic correctness or adversarial test integrity.

```sh
PYTHONPATH=apps/agent/src apps/agent/.venv/bin/python -m hickory_agent_worker.history_benchmark \
  --dataset docs/experiments/historical-code/2026-11-01/dataset.json \
  --references target/local-agent/history/fresh-references.json --verify
```

## Read the task in context

Requests contain prose and fenced code, not a JSON dump. The current method is followed immediately by the actual public-check failure, then read-only collaborators and a public example. Each repair request contains the original task, current candidate and current public feedback. Earlier attempts remain in the journal. Requests ask for a cohesive document with exactly one C# method block. Malformed replies are failed attempts; explanation quality is not independently graded.

Only whitelisted fields enter the request. The target reference answer, commit messages and held-out check bodies remain outside the model context. Some read-only collaborator methods also appear as targets in other cases; this is deliberate supplied context and makes the cases dependent. Public compilation never includes held-out check bodies, so compiler errors cannot disclose them. A truncated response is never compiled or executed. At public success, repair stops; held-out checks run once afterward and their feedback cannot guide another attempt.

JSON is retained for machine-readable artifacts and measurements. `--layout separate` moves the same human-readable blocks into a separate evidence section. The initial coding trial uses `literate`; it does not by itself estimate a layout effect or exercise the live Hickory action registry.

```sh
PYTHONPATH=apps/agent/src apps/agent/.venv/bin/python -m hickory_agent_worker.history_benchmark \
  --dataset docs/experiments/historical-code/2026-11-01/dataset.json \
  --references target/local-agent/history/fresh-references.json \
  --models qwen3-coder:30b qwen3.8:27b --iterations 2 --layout literate \
  --output target/local-agent/history/new-trial
```

Both models receive CPU-only execution, six threads, 8192-token context, 1536-token response limit, temperature 0.2, top_p 0.95, top_k 20, seed 42 and thinking disabled. Requests use native Ollama text output. Each task allows at most two attempts with public feedback. Exact requests, replies, timing phases, candidate code, failures, model digests and image identities are retained. Models run sequentially and unload between models. This is a fixed-budget comparison rather than per-model sampling optimization.

## Interpret the comparison

Compare model public/held-out success and first-attempt versus repaired success against the qualified historical method. Distinct correct implementations pass; textual similarity to the historical code is not the correctness score. Report elapsed time and tokens for model runs, but do not infer human work time from the spacing between commits. These histories contain no reliable measurement of time spent, false starts or original assistance.

There are eight tasks but only four source commits, including multiple closely related tasks. Do not treat them as eight independent samples of a programmer's ability. The tiny curated set, retrospective specifications, supplied collaborators and limited tests cannot establish general parity with the author. Public pre-2021 code may be in model training data; its age does not establish contamination freedom. No tuning split is established here, and further optimization against these cases would turn them into development data.

[Dataset and reference methods](../experiments/historical-code/2026-11-01/dataset.json), [source provenance and exclusions](../experiments/historical-code/2026-11-01/provenance.json), and [qualified reference receipts](../experiments/historical-code/2026-11-01/references.json).

## Experiment with debugger probes between model calls

This is a proposed extension, not an implemented batch debugger. The existing native bridge waits for a step to settle and records source/stop/frame-scoped expression receipts. The UI step action acknowledges dispatch before the stop arrives. An autonomous sequence must use a settled-stop acknowledgement and refresh scope between operations, rather than issue repeated UI clicks against one old handle.

Existing watches refresh at pauses, and adapter capabilities expose conditional breakpoints, hit conditions and logpoints. First test whether a model can install a few targeted probes, continue until an interesting stop and read one source-aligned trace. The primary hypothesis is that coherent source-aligned observations improve diagnosis and verified repairs; reducing repeated inference is secondary. Where stepping is necessary, allow a bounded plan: evaluate chosen expressions, step over at most N stops, collect value changes, and stop on a requested predicate, breakpoint, exception, source/frame change, cancellation or budget exhaustion. Each operation must retain the existing session permissions and freshness checks. Current readings bind frame handles to a stop; refresh them after stepping rather than reuse the initial handle. Following a call across stops needs an explicit continuity policy, and ambiguous continuity should stop the plan. Reverse steps require actual adapter support; browsing recorded earlier values does not reverse the running program.

Represent the plan and observations as sections in a literate document. Group expression changes and messages at the lines and stops that produced them; retain the full ordered receipts separately. A compact reading can show where an invariant first failed without making the model process an unrelated transcript or a JSON state dump.

First replay identical debugger receipts as source-aligned versus chronological readings, keeping facts and available actions fixed. Make verified repair success the primary outcome; count wrong-source and wrong-frame associations as diagnostic outcomes. Then compare one-model-call-per-step execution, fixed batches and bounded conditional probes in a separate interactive experiment with equal permitted operations and runtime budgets. Adaptive probes may collect different facts, so their effect cannot be attributed to placement alone. Report calls, tokens and elapsed time as secondary measurements. The historical coding trial above uses compiler/public-check feedback; it does not yet measure this debugger hypothesis. See the [factorial study protocol](../../.hickory-source/mechanisms/apps/agent/factorial-context-study.md) for feature interactions and confirmation requirements.
