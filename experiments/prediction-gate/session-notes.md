# Continue the prediction-gate evaluation

Saved November 4, 2026. This is a continuation summary, not a verbatim chat export. The user asked to preserve this thread and write a blog post recording an idea that did not pan out. The draft is [blog-post.md](blog-post.md); nothing has been published.

## Objective and decisions

Evaluate Hickory's existing prediction-before-verification mechanism against the same coding workflow without it. Preserve initial reasoning, feedback and revisions; separate public gate checks from evaluator-only held-out tests. Do not infer an internal mental model from behavioral performance.

The original three-task smoke suite was too easy. The user requested complexity based on nloum/LiveLinq, explicitly disregarding ComposableCollections, and identified DynamicCombineLatest in this repository. The frozen reactive-v4 suite has six deterministic Python semantic variants. This does not test native C#/Rust concurrency. See [reactive-plan.md](reactive-plan.md) and [README.md](README.md) for implementation, provenance, resource limits and reproduction.

Within-model conditions use identical starting code, instructions, tools and limits except the gate. Four repetitions × six variants × two conditions = 48 sessions per model. Primary outcome: accepted final program passes every held-out case. Positive success criterion: at least +10 percentage points and a paired-bootstrap lower 95% bound above zero. Keep pilots and simulated plumbing checks separate. Luna is an exploratory follow-up chosen after seeing Sol's ceiling results.

## Main conclusions

Sol baseline: first 24/24, final 24/24. Sol gate: first 24/24, final 22/24. Two correct-code prediction mismatches prevented acceptance. Difference −8.3 points, paired-bootstrap 95% interval [−20.8, 0], exact McNemar p=.50.

Luna baseline: first 16/24, final 16/24, 17 completed tasks. Luna gate: first 15/24, final 1/24, one completed task. Fourteen correct candidates were refused: twelve ledger prediction mismatches and two missing prediction fields. One incorrect candidate was also blocked. Five sessions had model response-format failures, retained in results. All 65 CLI turns completed with no provider refusals. Difference −62.5 points, interval [−79.2, −41.7], exact p=.0000610. No regressions recorded. Fixed variants and exploratory model selection limit generalization; do not pool model effects.

Mechanistic interpretation: exact large JSON traces add bookkeeping failure opportunities; immutable wrong predictions can conflict irreparably with the original requirement. This is an interpretation, not a diagnosis of the model's internal reasoning. Sol's distinct-ownership failure is a useful example: correct code and rationale said only z subscribed; structured prediction included y and z. Correct code was blocked. Production source editing can occur before verification; the mechanism is not a universal save barrier.

## Evidence locations

All paths below are relative to the repository root `/Users/family/Documents/src/hickory-docs`.

- Frozen pre-live source: `target/prediction-gate-reactive-v4-source/snapshot.json` and adjacent copies.
- Sol main: `target/prediction-gate-reactive-main-20261103-214715/`.
- Sol review: `target/prediction-gate-reactive-main-review-20261103-214715/review.json`.
- Sol evidence ZIP: `target/prediction-gate-sol-reactive-v4-evidence.zip`.
- Sol maintained report: `experiments/prediction-gate/sol-results.md` and `.json`.
- Luna pilot: `target/prediction-gate-reactive-luna-pilot-20261104-050751/` (not pooled).
- Interrupted Luna main: `target/prediction-gate-reactive-luna-main-20261104-053143/`; diagnosis in `target/prediction-gate-reactive-luna-main-review-20261104-053143/review.json`. Thirty-nine quota refusals; exclude as interrupted, retain as failure evidence.
- Completed Luna main: `target/prediction-gate-reactive-luna-main-retry-20261104-060132/`.
- Combined review: `target/prediction-gate-reactive-comparison-review-20261104/results.md` and `review.json`. Maintained copy: [comparison-results.json](comparison-results.json).
- Each live directory contains `manifest.json`, `records.jsonl`, `analysis.json`, `results.md`, per-session patches and ledger receipts, and `transport/` CLI receipts.

The combined review's raw-record SHA256 identifies the Luna retry. Manifests match frozen runner, fixtures and transport. Treat the raw generated report's generic “inconclusive” label cautiously: Luna shows a substantial negative effect on these fixed tasks. It is neither equivalence nor a general claim that prediction never helps.

`target/` is local and disposable. Retain the evidence archive outside cleanup before publishing or moving machines. A public post should link a durable, reviewed archive, not claim these paths are public downloads. Do not include credentials, personal CLI configuration or authentication files in a public release.

## Resume commands and constraints

Original main command (fresh output path required):

```sh
python3 experiments/prediction-gate/bench.py \
  --live --backend codex --suite reactive \
  --model gpt-6-luna --reasoning low --repeats 4 \
  --out "target/prediction-gate-reactive-luna-main-$(date +%Y%m%d-%H%M%S)"
```

The signed-in Codex backend uses subscription allowance, with no API fallback. Usage estimates can differ from billing. Running from this agent's shell previously failed managed-preferences initialization; successful live runs were launched by the user in Terminal. Do not repeat paid runs merely to polish prose.

Do not modify the frozen benchmark to improve a condition's result. A recovery policy, targeted predictions or another model/effort setting constitutes a separate follow-up. Candidate next experiment: isolate prediction's effect on initial patch correctness from the acceptance policy's effect on final correctness. This is proposed, not executed or approved as paid work.

Follow AGENTS.md: maintained changes belong in `.hickory-source/`, regenerate/check/audit, focused tests only. The reactive owner is `.hickory-source/mechanisms/prediction-gate-reactive-benchmark.md`. Preserve unrelated working changes. No broad regression monitoring, commits or publication were requested.
