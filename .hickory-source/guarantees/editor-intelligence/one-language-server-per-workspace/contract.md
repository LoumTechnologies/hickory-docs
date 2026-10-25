<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# One Language Server Session Per Workspace

For: Authors using completion, diagnostics and refactoring inside literate source.

Given the local server with several documents and plain files open across
any number of windows and panes, when any of them asks a language question,
then there is exactly one `hick-lsp` session for the workspace — and so one
rust-analyzer, one pyright — shared by every socket: each connection's
requests are answered under its own ids, notifications reach every
connection, a later connection is told the server's capabilities on
subscribing, a file two panes hold open is opened once with the server and
closed only when the last of them lets go, and a connection that drops
closes only what it was the last to hold.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/one-language-server-per-workspace.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
