<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Keep model reasoning distinct from the answer

For: Authors using an agent to change documents while retaining review control.

Inspect a recorded turn without mistaking a reasoning block for the final response or an executed check.

## What must happen

The conversation distinguishes reasoning, final response and tool observations using their recorded roles. The current Python runtime owns prompt/history construction; the shared session and UI retain the separate recorded content. Reasoning text does not execute tools, certify completion or establish a check result.

## Limits of the evidence

Provider-specific availability of reasoning is not a promise to expose hidden model state.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/reasoning-is-shown-apart-from-the-answer.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain role separation while applying the Python-runtime amendment. Earlier Rust-loop test claims remain historical.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
