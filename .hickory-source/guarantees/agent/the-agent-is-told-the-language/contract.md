<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Teach the agent the document syntax its tools will accept

For: Authors using an agent to change documents while retaining review control.

Ask the agent to produce an executable document. Its syntax guidance should match the supported elements instead of inventing attributes.

## What must happen

The current built-in Python runtime supplies document-language guidance and shared tool descriptions. That guidance identifies supported elements and their semantics, including the declared boundaries for diagrams and generated source. Host parsing and validation remain authoritative; a prompt is not an alternative grammar or permission boundary.

## Limits of the evidence

A documented syntax list does not guarantee that a model follows it or that every explanation is accurate.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/the-agent-is-told-the-language.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Move ownership of the prompt to the current Python runtime while preserving the requirement that the advertised document vocabulary be accurate.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
