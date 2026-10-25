<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Pair identity without silently granting execution

For: An engineer pairing their own machines and reviewing a lost machine's access.

Pairing the same key twice must not restore an execute grant that was revoked. Registering identity alone must not be presented as an established connection.

## What must happen

Each machine retains its private identity locally and pairs by exchanging public identity. Fleet membership is mutual. Grants are per machine; execution is initially off, pairing is idempotent, conflicting identities under one name are refused, and damaged invitations fail. Registering a key does not by itself establish a network connection. Optional peer connectivity is governed by the peer-channel contract.


## Limits of the evidence

The original contract also describes platform and invitation cases. Those clauses remain in historical evidence and require a full fleet review before the replacement can be accepted as exhaustive. This page is partially reviewed, not a certification of every pairing path.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/a-machine-is-a-keypair.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Scope nothing-is-reachable to identity registration; remove the obsolete implication that the product has no optional connectivity.

Implementation review: partially verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
