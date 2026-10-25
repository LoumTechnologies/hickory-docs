<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Reuse a recording only for the inputs that produced it

For: Reviewers deciding what an output, recording or prediction actually establishes.

Change an authored script without changing the shell command. The old recording must become stale; writing the transcript itself must not invalidate its own key.

## What must happen

A recording key includes the cell's command, relevant environment and features, declared seeded input bytes, and upstream cell identities. Literal document-produced inputs remain part of the digest. The document's own unstable outputs, weave targets and retained recording payloads do not key a cell on its own results. Seeded volume state is hashed before earlier cells mutate it; upstream outputs arrive through the upstream term. Unseeded output-only volumes contribute no input bytes. Cache directories are excluded at every depth. A broad output-path exclusion must not erase real inputs.

## Limits of the evidence

Correct identity coverage depends on declared inputs. A matching cache key does not establish correctness of an undeclared ambient dependency.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/a-recording-is-keyed-by-the-cells-inputs.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Combine the September 2 and 3 input-digest amendments, including the explicitly withdrawn output-volume-prefix shortcut, into one rule.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
