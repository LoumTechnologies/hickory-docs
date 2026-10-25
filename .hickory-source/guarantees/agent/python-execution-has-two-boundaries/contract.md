<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Separate model snippets from trusted host operations

For: Authors using an agent to change documents while retaining review control.

A model snippet cannot import a shell as a fallback. A separately registered terminal action can still run a host process under the existing host policy.

## What must happen

Model snippets execute in Monty without workspace mounts or a CPython/shell fallback. The trusted Python orchestrator runs under supported Seatbelt or Bubblewrap confinement; unavailable confinement refuses hosted execution. Python owns its conversation graph, prompts, retries, budget and completion attempts. Hickory supplies bounded transport, reviewed document edits, host operations and retained receipts. Registered window tools can expose execution, debugger, setup and filesystem effects under local policy. Stop cancels inference/review and terminates the confined process group; it does not approve a proposal or guarantee termination of an independent job already launched by a completed UI operation.

## Limits of the evidence

Trusted runtime installations and host policies remain assumptions. This is not an adversarial same-user security boundary. Standalone development has a different confinement boundary.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/python-execution-has-two-boundaries.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain both confinement layers and explicitly incorporate the later host-capability grant. Remove the implication that every possible workspace effect goes through document-edit approval.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
