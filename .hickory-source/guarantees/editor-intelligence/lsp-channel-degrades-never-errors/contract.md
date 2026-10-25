<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The LSP Channel Degrades To Structural Or Null Answers, Never Errors

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a doc WebSocket with the 0x02 LSP bridge (api.md v0.3), when a child
language server (pyright, rust-analyzer, …) or the hick-lsp binary itself is
unavailable, then every client request still receives a JSON-RPC response
with a `result` (structural copy/paste answers where the parser can provide
them, `null` otherwise) — never an `error` frame, a dropped request, or a
closed channel. Definition/reference targets inside generated outputs are
translated through the last run's provenance to `hick:///` source
coordinates when byte-mappable, else returned as `hick-output:///` targets.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/lsp-channel-degrades-never-errors.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
