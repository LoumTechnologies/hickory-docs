<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Report whether verification found drift, no baseline or a false expectation

For: Reviewers deciding what an output, recording or prediction actually establishes.

CI must distinguish a stale recording that needs regeneration from a cell with no baseline and from an expectation that actually failed.

## What must happen

Verification reports verified, drifted, unverifiable or expectation-failed with their distinct public exit codes. A stale recording counts as drift; a wholly missing baseline counts as unverifiable. A frozen valid recording is a baseline. Unverifiable cells do not evaluate expectations or supply trustworthy drift comparisons. When several outcomes occur, expectation-failed outranks unverifiable, which outranks drifted, which outranks verified; all findings remain visible.

## Limits of the evidence

Exit status summarizes evidence, not application correctness. A failed expectation needs a requirement/code decision rather than automatic baseline regeneration.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/test-separates-unverifiable-from-drifted.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Fold the stale-recording amendment into the four-outcome contract. Retain the public code mapping and severity ordering from the original detailed requirements.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
