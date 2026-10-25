<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Provider Key Entered in Settings Stays on This Machine

For: Authors using an agent to change documents while retaining review control.

Given the desktop app, when the user enters an LLM provider API key in the
Settings page, then the key is written to one JSON file under the app's own
config directory (`llm-keys.json`), owner-only (`0600`) on Unix, read by this
process alone, sent to exactly one place — the provider the user chose, on
that provider's own API — and never logged, echoed back over the local API,
or transmitted anywhere else. There is no telemetry to carry it and no server
of ours for it to reach; those do not exist in this product
(`docs/specs/freeform/local-only.md`).

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/keys-are-stored-locally-and-never-leave.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
