<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Keep a prediction separate from the answer it is tested against

For: Reviewers deciding whether an agent demonstrated an independent prediction rather than copying an observed answer.

Predicting 160 and observing 160 still fails a requirement of at most 100. Changing the prediction after observing the result cannot earn first-attempt credit.

## What must happen

A managed attempt is admitted against a sealed prediction with exact source, input, dependency, environment and target identities. Matching a prediction and satisfying the original requirement are separate checks. Observed, stale, failed, replayed or unknown attempts cannot receive current certification. Exposure history remains attached to the scenario; changing source, names or sessions does not silently erase it. Human inspection records exposure before returning private answers. Interrupted attempts remain Unknown; corrupt history is refused.


## Limits of the evidence

This is a concise review contract. Adapter-specific clauses from the 990-line original remain in historical evidence and the coverage-debt ledger until individually reconciled. Ledger success does not certify host isolation, browser freshness, publication or authenticated evidence against a same-user attacker.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/predictions-precede-managed-observation.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Extract the core observable promise; separate historical checkpoints from current contract and explicitly retain adapter review debt.

Implementation review: partially verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
