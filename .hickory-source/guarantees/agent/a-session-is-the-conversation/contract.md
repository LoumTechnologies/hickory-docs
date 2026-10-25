<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Read the retained branch of an agent conversation

For: Authors using an agent to change documents while retaining review control.

Reopen a conversation after changing branches. The recorded selected branch must remain available; it must not be reconstructed from a model summary.

## What must happen

The session retains prompts, responses, actions, observations and branch relationships as document evidence. The selected branch determines the conversation reading. Python owns current conversation/context construction; Rust provides supervision, tool boundaries, transport and recording. Abandoned work remains historical rather than silently becoming current context.

## Limits of the evidence

Opaque provider delivery and hidden model state are not reconstructed by a retained session.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/a-session-is-the-conversation.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Apply the Python-runtime amendment. Retain the existing session and branch vocabulary; retire references to the removed Rust ReAct loop.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
