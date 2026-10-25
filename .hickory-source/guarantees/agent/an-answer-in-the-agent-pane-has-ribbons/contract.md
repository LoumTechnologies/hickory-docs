<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Trace conversation passages without confusing context with support

For: Authors using an agent to change documents while retaining review control.

Follow a link from an answer to the unsaved editor snapshot supplied to that turn. The link must describe historical input, not claim that it proves the answer.

## What must happen

The live Agent pane uses the protected conversation editor over the ordinary document editor. Recorded and streaming content remains protected; the response region remains editable. Context links use the actual recorded branch and source snapshots, including identifiable untitled editor addresses. They can attach to answer paragraphs in the same user turn and remain visible through folded bookkeeping. A new user turn does not inherit the previous turn's input links. The pane follows normal workspace stacking so connections remain visible.

## Limits of the evidence

Context records what was supplied. It does not establish derivation, entailment or current source freshness.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Fold the live-document, explicit-buffer-identity, per-turn context and pane-stacking amendments into the current reading contract. Keep the standalone read-only session-card view separate.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
