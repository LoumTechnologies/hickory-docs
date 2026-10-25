<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Explicit UI action dispatch checks scope, state, policy and retries

For: Authors using an agent to change documents while retaining review control.

Given a command explicitly registered with `UiActions`, observing its parent
returns a bounded descriptor with subject, schema, current state and an opaque
action handle. A structured invocation through that registry checks workspace,
window, current actor policy, current revision, enabled state and exact typed
arguments before calling its registered semantic handler. Quoted text, source
location handles and forged handles do not execute commands. Observation identity
is distinct from executable identity and does not grant an edit baseline.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/ui-action-dispatch.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
