# Evaluate prediction gating without confusing it with correctness

For Hickory maintainers deciding whether to fund a controlled model experiment.

The local harness exercises the existing private-output runner and durable ledger.
It adds a host policy that accepts a benchmark save only after certification.
The production mechanism itself permits source writes before prediction checking;
calling it a universal “cannot save code” guardrail overstates its current scope.
A valid subscription-backed smoke pilot completed on 2026-11-03: both conditions
passed 3/3 tasks on the first proposal, with no mismatches or blocked saves. It
established harness operation but showed no correctness difference on those tasks.
The user then requested the separate [reactive-v4 benchmark](reactive-plan.md).
Simulated records remain plumbing evidence only; do not pool benchmark versions.

## What the existing implementation does

An agent predicts an observable value for a host-selected question, using an
`equals`, `number_at_most`, or nested `fields` predicate and a rationale. A target
can be one execution's output, a debugger expression at a source span, frame and
hit count, or both together. The rationale is retained text; the checker does not
score its quality. See [predicate and target definitions](../../crates/hick-observe/src/prediction_types.rs)
and [private session setup](../../crates/hickory-cli/src/serve/private_predictions.rs).

The host owns the scenario, concrete inputs, original requirement, actor and
captured environment. Sealing records exact SHA-256 identities for source,
inputs, dependencies and environment before execution admission. A repair needs
a new source-bound seal. The original prediction and requirement survive source
revisions and sessions within the same recognized scenario. Reusing an admission
returns its earlier result rather than reexecuting. The
[ledger classification](../../crates/hick-observe/src/prediction.rs) compares the
actual value separately against the sealed prediction and original requirement.
The [output runner](../../crates/hick-literate/src/prediction.rs) also requires
accepted execution evidence; matching a prediction cannot override a failed
`hick:expect`. Debugger and combined adapters share the gate.

A mismatch, failed original requirement, rejected execution, changed prediction,
changed requirement, stale identities, replay, unknown provenance, interruption,
or unavailable observation cannot earn current certification. Corrections that
retain the original expectation can pass with a distinct after-correction label.
Retained debugger coaching also distinguishes prediction before launch from an
informed prediction of the next stop after earlier state was shown.
[Promotion](../../crates/hickory-cli/src/serve/private_predictions.rs) is a separate
host action that rechecks current source/runtime and required debugger questions.

The [source edit tool](../../crates/hickory-agent/src/tools/mod.rs) writes edited
Markdown after parse/weave checks, before prediction verification. The
[private verify tool](../../crates/hickory-agent/src/tools/prediction.rs) returns
bounded summaries and publishes no generated outputs even on success. Ordinary
[verified publication](../../crates/hickory-agent/src/tools/verify.rs) is another
host policy and explicitly is not private prediction mode. Private service status
labels application integration incomplete. This inspection does not establish a
universal filesystem save barrier.

## Can the model obtain the answer first?

Through the admitted private tools, it can read/edit source, seal and verify.
[Dispatch](../../crates/hickory-agent/src/tools/dispatch.rs) refuses generated
output/file reads and alternate execution; private startup avoids ordinary cached
transcripts. Raw values, diagnostics and execution files are withheld, and the
host keeps ledger storage private. Managed inspection records exposure, and
changing scenario names/source/session does not legitimately reset known history.
These controls depend on the host's scenario correspondence and isolation.
They are not an OS sandbox against a same-user process. External tools, an
incorrectly confined ACP adapter, filesystem access, old conversations or leaked
host metadata can defeat the intended withholding; the repository itself records
integration and adapter evidence limits in the
[current contract](../../.hickory-source/guarantees/verification/predictions-precede-managed-observation/contract.md).

The status endpoint serializes the full host question, including its requirement
predicate; an equality requirement therefore exposes its expected value to any
allowed consumer of that status. Whether it reaches a particular model depends
on that host surface. The expected behavior can also be explicit in source `expect`
text, task requirements, examples or coach question wording. For example the
[private UI fixture](../../scripts/test-private-prediction-ui.sh) states the total
100ms budget and second allowance 20ms in its requirement references. That is a
source inspection finding, not proof of leakage in every question. The API does
not require an exact numerical forecast: a very permissive `number_at_most`
predicate is allowed. Repeating a requirement bound can earn a prediction match
without explaining the trajectory. `fields` enforces exactly the named field set,
but still permits broad subpredicates.

Repeated known tasks invite memorization. Small fixed input sets invite hardcoded
solutions. Pass/fail summaries are an information channel even when actual values
are withheld. Repairs can fit those cases, and changed concrete inputs create
independent scenarios. Unknown prior correspondence must remain unknown rather
than silently earning first-attempt credit. The ledger resists changing the
original expectation after feedback; it cannot establish understanding. A correct
implementation can also be blocked indefinitely by a mistaken original prediction.

## Frozen experiment v1

Freeze this document, `cases.py`, runner and bridge hashes before the first live
call. The manifest records them, revision, dirty status, settings, randomized
order and executable hash. Preserve a snapshot of the working revision when dirty.
Do not change cases, scoring, exclusions or success criteria after live outcomes.
Repairs to experiment bugs require a new version and rerunning both conditions;
retain failed version records. The current task oracles are fully enumerated,
not randomly generated during scoring.

Primary outcome: the fraction of all scheduled sessions whose final accepted
implementation passes every held-out test. All scheduled failures, timeouts,
abandonments and protocol errors remain in the denominator. Use paired gate minus
baseline risk difference across the same task/repeat. Predefined success requires
an improvement of at least 10 percentage points and a paired bootstrap 95% lower
bound above zero. Otherwise report inconclusive, unless a separately designed
noninferiority/equivalence study can support a claim of no useful benefit.

Report raw numerators/denominators, first proposed patch correctness before any
test or submit feedback, completed tasks, regressions against initially passing hidden
checks, prediction mismatches, blocked saves, retries, abandonments, and both
false passes and false blocks relative to held-out correctness. “Completed” means
an accepted submission passes public checks; it does not imply held-out success.
First-submission correctness is recorded separately because a submission may
follow ordinary test feedback; transcripts disclose that exposure. A test action cannot certify or save code.

Three tasks cover a retry budget boundary across helper/caller functions, inventory
reservations with failed-operation state preservation, and cache expiry/replacement
with lookup/cleanup agreement. Each has buggy starting code, explicit requirements,
1–2 public cases and four held-out cases. The fixtures also specify initial
regression cases. Public and held-out inputs do not overlap. The small deterministic
suite is intentionally inspectable by evaluators and weak against memorization.

Both conditions get identical task text, starting code, model, temperature 0,
JSON tool protocol, restricted Python subset, public cases, six calls, 2,048 output
tokens per call, a cumulative 32,000 UTF-8 input-byte cap and 180-second loop budget.
Provider HTTP calls time out at 30 seconds, gate operations at 15 seconds and
candidate executions at 3 seconds; a session can overrun its loop deadline by one
in-flight operation. Input bytes conservatively budget tokens; provider usage
counters supply measured tokens. Missing usage and cost are null, never zero.

The baseline can test and submit, sees public case-group pass/fail, and can revise on the same
call budget. Its submissions save even on public failure. The gate additionally
requires a sealed exact result-array prediction and rationale before every test
or submission execution, returns the existing runner's bounded summary, and accepts saves only
on production ledger promotion. Both see the same public case-group pass/fail; the gate
also exposes its prediction status. Gate overhead counts against the shared time
budget. Asking for a prediction necessarily changes that instruction and response
payload; there are no other intentional differences.

Each arm is a fresh HTTP conversation and directory; no previous arm history is
sent. Execution order is shuffled with seed 20261103 across all task/repeat/arm
entries. Model weights/provider caching and external provider state cannot be
reset by this harness. Pair labels are replication blocks, not matched random
samples from the provider. Keep provider/model version fixed and record changes.
There is no shell, repository or file tool in either arm. Only public JSON context
is sent. Candidate AST admission excludes imports, reflection, private names and
arbitrary attributes; execution uses a separate interpreter with a timeout.
This restricted trusted-fixture evaluator is not suitable for hostile arbitrary
programs or an unconstrained coding agent.

Held-out cases, solutions, expected values, scores and host diagnostics stay in
the evaluator process/files and never enter model context. The evaluator imports
the fixture module; the model receives only selected public fields. A candidate
has no supported filesystem access. Stored trial artifacts contain held-out
answers: keep the entire output directory evaluator-only during future runs.
This harness evaluates the production **output adapter**, not debugger coaching,
ACP, editor UI, arbitrary projects or the complete ordinary AI coding workflow.
Generalizing to those surfaces requires a separate integrated experiment. The
user's requested harness is an explicit experiment; it does not replace the
in-app opt-in measurements for indexes and lineage.

## Trial count and cost before live use

First run the simulated pilot below. Then run a separate three-pair live plumbing
pilot (one repeat of each task) to check provider protocol/accounting; retain it
as pilot-only evidence. Main experiment: eight
repeats per task, 24 pairs, 48 independent sessions. Keep pilot and main output
directories separate. This small study estimates performance on three tasks;
replicates do not create 24 independent task types or strong generalization power.

At illustrative rates $2/M input and $8/M output, the conservative main cap is
48 × (32,000 × 2 + 6 × 2,048 × 8) / 1,000,000 = **$7.79**; a three-pair live pilot
adds at most **$0.97**, excluding provider-specific billing extras. These are
budget assumptions, not verified prices for any named model. Choose a fixed model
and replace the rate flags with its actual rates before approval or execution.
No credentials are read or paid requests made in simulated mode. Live mode is
explicit and requires a configured endpoint, model and credential variable.

## Reproduce

From the repository root, build only the experiment adapter and run focused checks:

```sh
cargo build --locked -p hick-literate --example prediction_bench
python3 -m unittest discover -s experiments/prediction-gate -p 'test_bench.py' -v
python3 experiments/prediction-gate/bench.py --simulated --out target/prediction-gate-pilot
```

The output directory must be new. It retains `manifest.json`, `records.jsonl`,
`analysis.json`, `results.md` and per-session initial/final code, final patch,
full contexts, each proposal patch, original committed prediction, production
ledger, check results, revisions and final held-out results. A gate commitment
file is written before any observations. The production ledger provides durable
ordering; the host records are review artifacts, not tamper-proof attestations.
There is no raw provider chain-of-thought capture; rationale is whatever the
model elects to return.

A compatible provider must implement chat-completions JSON requests and return
`choices[0].message.content` plus `usage.prompt_tokens/completion_tokens`. The
bridge uses the existing Rust private-output runner; no Python replica decides
whether a prediction matches. With the chosen provider/model and real rates:

```sh
export BENCH_API_KEY='YOUR_KEY'
python3 experiments/prediction-gate/bench.py --live \
  --provider-url "$BENCH_PROVIDER_URL" --model "$BENCH_MODEL" \
  --input-rate 2 --output-rate 8 --repeats 1 --out target/prediction-gate-live-pilot
# After validating the pilot protocol, use --repeats 8 and a new main directory.
```

A live model is not assumed available merely because the local Codex session
works. A provider protocol failure is retained, not automatically retried or
replaced with a simulated response. Hidden scores are never returned as repair
feedback. During a run inspect only evaluator-owned artifacts; do not feed them
back to the agent. Regenerate maintained files with `behavior.py generate`, then
`check` and `audit`. Broad regression and post-commit monitoring belong to the
maintainer (`just checks-status`).

## Use subscription allowance through signed-in Codex (protocol 3)

This macOS transport uses `codex exec` with your existing ChatGPT sign-in. It
forces ChatGPT authentication and removes API-key overrides from the child
process; it does not extract or repurpose OAuth tokens. No API key is required.
See [signed-in automation](https://learn.chatgpt.com/docs/non-interactive-mode)
and [subscription usage and credit rates](https://learn.chatgpt.com/docs/pricing).
Included usage and available ChatGPT credits follow your account's limits; this
transport does not claim access to your balance or permission to buy more credits.

From a normal Terminal with working provider connectivity:

```sh
codex login status
python3 experiments/prediction-gate/bench.py --live --backend codex \
  --model gpt-6.1-sol --reasoning low --repeats 1 \
  --out target/prediction-gate-subscription-pilot
```

If login status does not say ChatGPT, run `codex login` first. The output directory
must be new. Verify the three-pair live pilot for authentication, valid JSON,
usage counters, absence of native tools and no infrastructure failures before
running eight repeats in a separate main directory. Keep these records separate
from API-v1 and simulated runs. DNS/HTTP failures are infrastructure failures,
never evidence about prediction efficacy.

Protocol 3 repairs the protocol-2 transport parser; tasks, held-out oracles,
paired sample size and success criteria remain unchanged. The prior live pilot
is retained as an invalid infrastructure run, not reused as valid trial evidence. Both arms use the same explicit
model, reasoning effort, CLI version, wrapper instructions and confinement. Each
response starts a fresh ephemeral thread in an empty directory and receives only
replayed public conversation text. User config, repository instructions, memory,
apps, browser, shell, web and agent tools are disabled. Native tool/item events
invalidate the trial. macOS Seatbelt additionally denies reads of the repository,
receipts, temp roots containing other evaluator copies, and Codex session/history
files; the empty scratch directory is readable. Focused tests demonstrate denial
of known held-out paths. This is a scoped experiment quarantine, not a claim of
universal hostile-process isolation.

The CLI adds its own instruction/tool overhead and cannot apply v1's temperature
or 2,048-generation-token cap. Protocol 3 instead shares a six-turn limit, 30-second
transport timeout per turn, 180-second session loop budget, 32,000-byte replayed
input cap and a 16,000-byte accepted final-answer limit across arms. The answer
limit is checked after generation; it does not cap hidden reasoning or billing.
Terminating a timed-out CLI process is not a guarantee that provider processing
stops immediately. Provider-visible token usage includes CLI overhead and is
recorded from its completion event. Missing counters remain null. CLI turn attempts
are counted separately; opaque internal model-call/retry counts remain unknown.

At the published standard GPT-6.1 Sol rates (50 input, 2.5 cached input, 250 output
credits per million tokens), a planning scenario of 20,000 uncached input and
4,000 output tokens per turn is 2 credits/turn: up to 72 credits for 36 pilot
turns, or 576 credits for 288 main turns. This is an estimate, not an enforced
credit cap or a promise about included allowance. The actual turn count is often
smaller. No purchases or automatic API fallback are performed. `estimated_credits`
uses observed input/cache/output counts for this model; dollar cost remains null.
Source the published rate date when reporting, because rates can change.

### Protocol-2 pilot incident, 2026-11-03

The run in `target/prediction-gate-subscription-pilot-20261103-200027` completed
six real Codex turns, then rejected all six before admitting their proposals.
Codex 0.166 encodes the expected unstable-feature and disabled-Code-Mode startup
warnings as `item.completed` entries whose item type is `error`. The parser
mistook them for native tool activity. Its zero correctness counts and zero-width
interval are unusable as effectiveness evidence. No gate comparison ran.
The original directory remains unchanged; no answers are retroactively scored.

Raw completion events retain 52,509 input tokens and 854 output tokens (no cached
input reported), approximately 2.84 credits at the documented standard Sol rates.
That is an estimate, not a verified account debit. The old trial rows lost these
counters when parsing raised; the repair preserves available usage on rejected
turns. Protocol 3 accepts only the two identified startup diagnostics, retains
them in receipts, and continues to refuse unknown errors, failed turns and actual
native tool items. Infrastructure-only runs now receive an INVALID RUN label and
null effectiveness estimates. Parser integrity checks exercise warning admission,
native-tool rejection and usage retention without loading/scoring model answers.
Rerun both conditions in a fresh directory; do not combine this pilot with the
new one or proceed directly to the main experiment.

## Analysis and reporting limits

`results.md` reports counts and the paired risk difference. `analysis.json` adds
10,000 seeded paired percentile bootstrap samples and a two-sided exact McNemar
p-value from discordant pairs. Report all failures and per-task breakdowns from
JSONL beside those numbers. Percentile bootstrap intervals can collapse with
few/no discordances, especially in a tiny pilot; a zero-width interval then is
not proof of equivalence. Treat tasks as fixed; repeated-run intervals do not
estimate uncertainty over new task populations. Model API nondeterminism persists
at temperature zero. Token comparisons include prediction/context overhead.

False-pass means a certified proposal fails any held-out case; false-block means
a refused proposal passes all held-out cases. Those operational labels are
relative to this suite, not universal semantic judgments. Final accepted code can
differ from final proposed code. Compare first submission, revision sequence and
accepted final results to separate initial reasoning from feedback-assisted repair.
An incorrect original expectation can permanently block a later correct patch;
keep that failure rather than quietly resealing a new scenario.

Passing this benchmark demonstrates behavioral performance on these tasks. It
cannot prove the model possesses a mental model, establish causal cognitive
mechanisms, or show that real Hickory users benefit on representative work.

## Blog-post outline

1. Hypothesis: committing observable forecasts may improve implementation
   correctness; explicit alternatives are extra deliberation, filtering and test feedback.
2. Implementation: show seal → admission → withheld comparison → promotion,
   distinguish source saves from certification, and show requirement/prediction separation.
3. Experiment: publish the frozen hashes, task population, randomization, matched
   budgets, tools, held-out separation, sample size, prices and primary criterion.
4. Results: lead with real sample size and raw counts; show final/first-attempt
   accuracy, paired difference/uncertainty, costs, regressions and every failure.
   Keep simulated plumbing receipts in a separate clearly labeled appendix.
5. Examples: a retry-budget repair with unchanged prediction, an overfit constant
   that passes public checks but fails held-out tests, and correct code blocked
   by a mistaken original prediction. Use archived patches/receipts, not invented runs.
6. Limitations: three fixed Python tasks, constrained tools, visible requirement
   clues, provider state, feedback and selection effects, output-adapter scope,
   weak task generalization, and behavior versus claims about understanding.
7. Next study: preregister new tasks and integrated Hickory sessions before
   looking at their results; never retrofit success criteria to the winner.
