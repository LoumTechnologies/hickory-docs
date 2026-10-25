<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Finish an agent turn against its captured project boundary

For: Authors using an agent to change documents while retaining review control.

An accepted final response must explain the actual A-to-B change, including dirty source and editor identities captured at the start.

## What must happen

Completion capability and project boundary are negotiated explicitly. Accepted artifacts are checked against the captured starting state and completed bytes. The current hosted Python path requires a current independently accepted changeset; model declarations do not bypass the host check. Changed editor identities, save/rename transitions or destination state require a new valid boundary. Source transition and maintained-source incorporation remain separate checks.

## Limits of the evidence

Filesystem snapshots are observed reads rather than a global transaction; equality does not establish runtime behavior.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/changeset-completion-is-negotiated-and-checked.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Apply the current Python completion requirement and retire the older statement that built-in/Python mandatory finalization is not wired.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
