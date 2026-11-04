# I asked an AI to predict its code before accepting it. The gate rejected correct fixes.

I had a plausible idea for an AI coding guardrail: before accepting a change, require the AI to predict what its code will do. Run the code privately, compare the result with the committed prediction, and refuse the change when they disagree.

The hope was that predicting execution would force a more careful read of the code. An AI that could produce a convincing patch but could not follow its consequences might catch its own mistake before the change was accepted.

I tested that idea in Hickory Docs. On a small, frozen benchmark, it did not improve accepted-task correctness. With the stronger model, it blocked two correct implementations. With the cheaper model, it refused fourteen correct implementations and accepted only one task out of twenty-four.

This is a record of the mechanism we tested, what happened, and what the experiment leaves unanswered.

## What the guardrail actually does

The implementation is more specific than “the AI cannot save until it understands.” Source editing and verified publication are separate operations. Hickory's source-editing tool can write a document before prediction verification; private verification and later certification govern a different boundary. Calling this a universal save barrier would overstate it.

For the experiment, a submitted candidate counted as saved only when the gate accepted it. The AI had to provide its code, an explanation, and an exact JSON prediction of the outputs for two public input traces. The host committed that prediction before executing the candidate privately. Acceptance required both agreement with the prediction and satisfaction of the original public requirements. The original prediction remained fixed through subsequent revisions.

The model saw the behavioral contract and public inputs, so the expected behavior could be derived from its instructions. It could not see the held-out tests or their answers. Its explanation was recorded but not graded. A persuasive explanation could not compensate for a wrong structured prediction.

The production ledger supports several kinds of predicates. This experiment exercised exact output equality. Its findings apply to that demanding policy; they should not automatically be attributed to every possible prediction gate.

The implementation review and code links are in the [experiment README](README.md). The relevant mechanisms include [private verification](../../crates/hickory-agent/src/tools/prediction.rs), the [prediction ledger](../../crates/hick-observe/src/prediction.rs), and the [source-editing tools](../../crates/hickory-agent/src/tools/mod.rs).

## A comparison we could reproduce

We first checked the harness with explicitly simulated responses, then ran live pilots. Simulated results were plumbing checks, not evidence for the hypothesis. After freezing the reactive benchmark, we ran a main comparison using `gpt-6.1-sol` at low reasoning effort. Its initial patches were all correct, so we subsequently chose `gpt-6-luna`, also at low effort, as an exploratory follow-up.

There were six task variants, four repetitions of each, and two conditions per model: 48 sessions for Sol and 48 for Luna. Execution order was randomized. Sessions were independent and used isolated candidates and ledgers. Within each model, starting code, task contracts, tool access and resource limits were the same except for the gate.

The baseline could use ordinary public test feedback. Both conditions had up to six calls and six minutes per session, along with the same context, answer-size and execution limits. We retained initial patches, predictions, checks, feedback, revisions and final patches. This let us distinguish initially better code from changes made after feedback.

The tasks covered dynamic combination of event streams, flattening aliased collections with changing offsets, persistent groups, distinct-value ownership, and filters whose predicates depend on an item's index. They included removal, disposal, empty collections, retained state, callback ordering and interactions between functions. The kernels were Python semantic extractions informed by Hickory's `DynamicCombineLatest` and my LiveLinq code. They were not tests of native C# or Tokio scheduling, races or wakers.

Each task had two public traces used by the gate and twelve separate held-out traces used only by the evaluator. The primary outcome was the proportion of final accepted implementations passing every held-out case. We also scored first proposals, counted refusals and failures, and recorded tokens, CLI turns and elapsed time.

Before the live reactive runs, we defined success as an improvement of at least ten percentage points with a paired bootstrap 95% lower bound above zero. We did not change the tasks or that criterion after seeing the outcomes. The [frozen plan](reactive-plan.md) documents the protocol and exclusions.

## What happened

| Model | Condition | Correct first proposals / sessions | Final accepted correct |
|---|---|---:|---:|
| Sol | Baseline | 24/24 | 24/24 |
| Sol | Prediction gate | 24/24 | 22/24 |
| Luna | Baseline | 16/24 | 16/24 |
| Luna | Prediction gate | 15/24 | 1/24 |

For Sol, all 48 first proposals passed every held-out test. The gate blocked two correct implementations because their predictions were wrong. Both sessions then stopped without an accepted save. The final difference was −8.3 percentage points, with a paired bootstrap 95% interval from −20.8 to zero and an exact McNemar p-value of .50. This run exposed two concrete acceptance failures, but it was too small and too close to a ceiling to establish general harm.

Luna produced more coding errors and a much larger acceptance gap. Baseline accepted seventeen tasks, sixteen of which were correct on every held-out case. The gate accepted one task, which was correct.

The gate recorded fifteen blocked saves. Twelve were correct implementations whose committed predictions mismatched execution. One was an incorrect implementation with a mismatching prediction. Two more correct candidates omitted the required prediction field and were refused before a ledger comparison. These last two count as workflow failures, not evaluated prediction mismatches.

Six Luna gate sessions and three baseline sessions immediately finished without proposing code. The records do not establish why. Five sessions had response-format failures: three malformed JSON responses and the two missing predictions. We retained these outcomes rather than repairing or excluding them after the fact.

Luna's final difference was −62.5 percentage points, with a paired bootstrap 95% interval from −79.2 to −41.7 and an exact McNemar p-value of approximately .000061. There were fifteen pairs favoring baseline and none favoring the gate. This is evidence of worse accepted-task performance on these fixed tasks under this workflow. It is not a population estimate for arbitrary coding tasks.

Neither model met the predefined positive success criterion. We keep their results separate rather than pooling them into one effect.

One earlier Luna main attempt hit the subscription usage limit. We preserved it as an interrupted run and reran the entire comparison in a fresh directory. Its outcomes and both models' pilots are excluded from these main results. All 65 CLI turns in the completed Luna retry finished without provider refusals.

## A correct explanation, correct code, and a wrong prediction

One Sol failure concerned the final subscription list in the distinct-ownership task. After a sequence of attachments and detachments, only `z` should remain subscribed.

The code returned the correct result and passed all twelve held-out cases. The accompanying explanation also said that only `z` remained. But the structured prediction included both `y` and `z`.

The host therefore reported that the original requirement was satisfied but the prediction did not match. The fix was not accepted.

That discrepancy is observable. Its internal cause is not. It could reflect a transcription error, incomplete reasoning, or another failure. The explanation alone cannot settle which. What we can say is that the gate turned a discrepancy in the prediction artifact into rejection of correct code.

## Why I think this happened

My strongest interpretation is that we added a difficult second task without materially improving the first one.

Producing an exact trace means keeping event ordering, intermediate snapshots, group identities and subscription state consistent across a large JSON response. That requires execution reasoning, but it also requires accurate bookkeeping and serialization. Every extra field creates another way to reject a candidate that implements the contract correctly.

The initial-code numbers matter here. Luna started with sixteen correct proposals in baseline and fifteen under the gate. Its large final gap arose chiefly at acceptance. The results do not show a corresponding improvement in initial coding from asking for predictions.

The immutable prediction also creates a recovery problem. If the original requirement demands output A and the sealed prediction demands a different output B, no revision can satisfy both exact equality checks. Feedback cannot repair that contradiction without a policy change. Some sessions stopped after receiving a mismatch; the policy itself had left no successful code revision available.

Those are explanations of this workflow's behavior, not proof about the models' internal understanding. The broader idea that making predictions can improve reasoning remains separable from this particular refusal policy.

## The extra work had a cost

| Model | Total tokens, baseline → gate | Elapsed time, baseline → gate |
|---|---:|---:|
| Sol | 251,575 → 287,103 | 7.6 → 12.7 minutes |
| Luna | 328,779 → 392,466 | 6.9 → 8.9 minutes |

These totals cover all sessions in each condition, including failed responses. Sol's gate used 14.1% more tokens and 65.7% more elapsed time. Luna's gate used 19.4% more tokens and 27.5% more elapsed time. CLI turns increased from 24 to 26 for Sol and from 28 to 37 for Luna. Provider-internal call counts were unavailable.

Using the recorded standard rates and cached-token counts, estimated subscription usage was 12.22 versus 17.55 credits for Sol and 0.615 versus 0.758 credits for Luna. These are usage estimates, not account billing receipts.

## What I would not claim

There were only six fixed task variants, and two shared the same underlying stream-combination kernel. Repeating them adds observations of model behavior; it does not add new algorithms. The bootstrap intervals and paired tests describe these observations under their statistical assumptions, not uncertainty over the whole space of software work. Luna was an exploratory follow-up chosen after seeing Sol's ceiling performance.

We also have little evidence about the gate's ability to filter incorrect code. Sol submitted none. Luna had only one admitted incorrect gate proposal, which was blocked. No false acceptance was observed, but that is a very small opportunity to observe one.

Known contracts and recurring public examples can reward reproducing expected answers, whether or not the model accurately follows a changed implementation. Broad predicates can be satisfied with limited reasoning. Conversely, exact large traces can penalize bookkeeping errors. Passing either kind of check would demonstrate behavior, not prove possession of a mental model.

The held-out checks measured these deterministic semantics. They did not measure native concurrency, asymptotic performance, maintainability or production reliability.

## What I am keeping from the idea

I would separate two questions in any future experiment: does committing a prediction improve the initial implementation, and does refusing acceptance based on that prediction improve the final result?

Possible changes include smaller targeted predictions or a recovery policy that allows a new commitment after feedback. Those would be new interventions, requiring a newly frozen experiment. We have not tested them here.

For this implementation, the result is negative: the prediction requirement did not deliver the predefined improvement, and correct-code refusals dominated the cheaper model's outcome. That is worth recording before changing the mechanism or moving on.

The [Sol report](sol-results.md), [continuation record](session-notes.md), and [combined machine-readable review](comparison-results.json) preserve the evidence and its limitations. The [benchmark README](README.md) and [protocol](reactive-plan.md) document reproduction. Raw local trial paths are listed in the continuation record; they must accompany any public evidence release because `target/` artifacts are not a durable public archive.
