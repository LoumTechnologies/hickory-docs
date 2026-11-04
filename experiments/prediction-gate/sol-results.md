# Sol reactive-v4 results: correct code blocked by incorrect predictions

For the author writing the prediction-gate blog post. This is the completed Sol
main comparison, not a pilot or simulated result. The declared success criterion
was not met. The gate showed no first-attempt coding improvement and refused two
implementations that passed every public and held-out check.

## Main outcome

The 24 paired observations comprise six fixed task variants with four repetitions
per variant: 48 isolated sessions using `gpt-6.1-sol`, low reasoning effort,
Codex CLI 0.166.0 and the frozen reactive-v4 limits. Both conditions used the same
starting code, repair contract, public inputs, tool restrictions and budgets;
the gate added prediction/sealing and acceptance requirements. The order was
randomized before execution. Pilots and the earlier invalid run are not pooled.

| Measurement | Baseline | Prediction gate |
|---|---:|---:|
| Sessions | 24 | 24 |
| First proposals passing all held-out cases | 24/24 | 24/24 |
| First-proposal held-out cases passed | 288/288 | 288/288 |
| Final accepted implementations passing all held-out cases | 24/24 | 22/24 |
| Final-state held-out cases passed | 288/288 | 266/288 |
| Completed tasks | 24 | 22 |
| Incorrect predictions | not applicable | 2 |
| Blocked saves of correct proposals | not applicable | 2 |
| Resubmission retries | 0 | 0 |
| Abandoned tasks | 0 | 2 |
| Regressions against initially passing cases | 0 | 0 |
| Completed CLI turns | 24 | 26 |
| Infrastructure/protocol failures | 0 | 0 |

The primary paired difference is **−8.33 percentage points** in final accepted
correctness (gate minus baseline). There were zero gate wins and two gate losses.
The predefined paired percentile bootstrap 95% interval is **[−20.83, 0.00]
percentage points**; the two-sided exact McNemar p-value is **0.5**. The required
+10-point improvement with a lower interval bound above zero was not achieved.

Those statistics describe this small fixed task set. Both losses occur in one
task variant, and the two DCL variants share code. Repetitions do not create new
algorithm families. With perfect observed baseline performance, the bootstrap is
also at a boundary: its zero upper endpoint does not prove that no benefit is
possible elsewhere. This is observed harm in these two sessions, not statistically
conclusive general harm or an equivalence finding.

## Where the two failures occurred

| Task variant | Baseline final | Gate final |
|---|---:|---:|
| DCL queued quiescence | 4/4 | 4/4 |
| DCL readiness and retention | 4/4 | 4/4 |
| LiveLinq distinct ownership | 4/4 | 2/4 |
| LiveLinq flattening aliases/offsets | 4/4 | 4/4 |
| LiveLinq index-sensitive filtering | 4/4 | 4/4 |
| LiveLinq persistent groups | 4/4 | 4/4 |

Both refused proposals correctly implemented the distinct-ownership kernel. Both
passed the public requirements and all twelve held-out traces. Their sealed JSON
predictions differed from the expected output in exactly one place: the final
subscription list in the second public trace.

```text
Predicted final subscriptions: ["y", "z"]
Required and produced result:  ["z"]
```

The trace detaches `y`, later attaches `z`, and finally detaches `x`. Both supplied
rationales correctly described that sequence and said that only `z` remained.
The structured predictions nevertheless included `y`. This is an observed
inconsistency between code, explanation and prediction; it does not identify the
model's internal cognitive cause.

The production ledger classified both attempts as `mismatch`, with
`original_requirement_satisfied=true` and `prediction_matched=false`. Promotion
was refused. Both sessions then returned `finish`; one explicitly explained that
its original prediction could not be changed after observation. Neither revised
or resubmitted the code. The accepted final file therefore remained the original
faulty starting implementation, rather than the correct proposed repair.

For this benchmark, the requirement and prediction each assert an exact output.
Once those two asserted values disagree, no code can satisfy both on that input
while preserving the original prediction. The observed abandonment is consistent
with that policy. It should not be reported as an inability to write correct code:
the model had already produced correct implementations by the benchmark's checks.

There were zero observed cases of incorrect code passing the gate, but there were
also **zero incorrect first proposals**. Therefore this run cannot estimate how
well the gate catches incorrect implementations. Its two false blocks are measured;
its incorrect-code detection sensitivity is unmeasured.

## Resource usage

| Measurement, totals per condition | Baseline | Prediction gate |
|---|---:|---:|
| Input tokens | 236,672 | 260,021 |
| Cached input tokens (included above) | 70,272 | 46,848 |
| Output tokens | 14,903 | 27,082 |
| Total input + output tokens | 251,575 | 287,103 |
| Sum of session elapsed times | 458.10 sec | 759.21 sec |
| Mean elapsed time per session | 19.09 sec | 31.63 sec |
| Estimated subscription credits | 12.22143 | 17.54627 |

The gate used **14.1% more total tokens**, **81.7% more output tokens**, and
**65.7% more elapsed time**. It consumed two additional CLI turns for the
post-refusal finish responses. These are descriptive differences for this run,
including prediction text, gate execution and the extra turns; they do not
separately identify the cost of each mechanism.

The credit estimate is **43.6% higher**, partly because the baseline received more
cache hits. At the same recorded rates with all input treated as uncached, the
estimates would be 15.55935 versus 19.77155 credits, a **27.1%** difference. This
normalization is supplementary context, not a revised primary outcome. Estimates
use the protocol's standard Sol rate assumptions; they are not verified account
debits. Actual internal provider request counts remain opaque: CLI turns and
measured token counters are reported instead of pretending those counts are known.

## Validity and evidence

The run directory is
`target/prediction-gate-reactive-main-20261103-214715`. It retains the manifest,
JSONL trial records, each initial/proposed/final implementation, every sealed
prediction and check result, private gate ledgers and all CLI event receipts.
The companion [machine-readable report](sol-results.json) records validation,
aggregate measurements, the predefined analysis and the raw JSONL digest.

The fixture module, runner and Codex transport hashes match the source snapshot
created before reactive live outcomes in
`target/prediction-gate-reactive-v4-source`. The production adapter binary also
matches. All 48 scheduled sessions and all 24 pairs are present. All submissions
were scored on twelve held-out cases; the six task variants therefore contribute
288 checks per condition, with repetitions reusing the same fixed cases.
No held-out/solution fields appeared in model-facing conversation records. CLI
receipts contain no native tool activity or unknown errors. All gate proposals
had a source-bound seal; passing proposals received promotion, and the two
mismatches did not. The 50 CLI turns completed without infrastructure failures.
These checks support this scoped protocol, not an adversarial audit of provider
internals or a universal host-isolation guarantee.

The original mechanism review, protocol, source links and reproduction commands
remain in [the runbook](README.md) and [the reactive plan](reactive-plan.md).
Do not edit the frozen tasks, limits or success criterion to make this result
look positive. Keep the raw held-out material evaluator-only during any further
model comparison; publishing results is a separate action.

## Blog-post outline with the actual result

1. **Hypothesis.** Committing observable behavior predictions before acceptance
   might improve code quality or catch mistakes. Separate those possibilities
   from claims about an AI possessing a mental model.
2. **What was built.** Explain the production seal/observe/promote ledger and
   withheld observations. Clarify that source edits themselves are permitted;
   this harness gates acceptance. The actual predicate API permits broad bounds,
   while this benchmark deliberately asks for exact complete public traces.
3. **Experimental design.** Describe the frozen matched baseline/gate conditions,
   isolated sessions, randomized order, identical budgets and held-out cases.
   Disclose the initial invalid parser run, the two valid pilot ceilings and the
   user-requested reactive task population. Keep pilots separate from main.
4. **Result.** All 48 first proposals passed held-out tests. Final accepted success
   was 24/24 versus 22/24. Present the −8.33-point difference, interval, p-value,
   raw counts and the fixed-task uncertainty limits together.
5. **Concrete failure.** Show the distinct-ownership trace, correct proposed patch,
   predicted `["y","z"]`, actual `["z"]`, correct prose explanation, mismatch
   receipt and subsequent finish. Explain why this exact-output gate could not
   recover through code changes while retaining the mistaken original prediction.
6. **Cost.** Show +14.1% total tokens and +65.7% elapsed time, plus raw input/cache/
   output counts and qualified credit estimates. Include the extra finish turns.
7. **What remains unknown.** No incorrect initial code was generated, so detection
   sensitivity was not exercised. These are small Python semantic extractions of
   LiveLinq/Hickory mechanisms, not native Rx.NET/Tokio or large-project trials.
   Repeated variants and shared DCL code limit generalization. Correctness on this
   suite does not prove understanding, and a wrong structured forecast does not
   by itself prove lack of understanding.
8. **Next study.** A Luna follow-up was suggested after the Sol pilot ceiling.
   Use the same frozen tasks and budgets, compare Luna baseline against Luna gate,
   and disclose that model choice as exploratory. Do not pool it with Sol or add
   harder cases until a separate version is registered. A future nonblocking-
   prediction arm could distinguish deliberation effects from rejection policy;
   it was not part of this experiment.

A defensible headline is: **“The AI fixed every task. The prediction gate still
blocked two correct fixes.”** A defensible conclusion is that this run found a
costly acceptance failure mode and no demonstrated correctness benefit, while
leaving broader effectiveness and cognitive explanations unresolved.
