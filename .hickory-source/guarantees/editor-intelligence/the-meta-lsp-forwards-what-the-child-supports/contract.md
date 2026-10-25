<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Meta-LSP Forwards Whatever The Child Supports

For: Authors using completion, diagnostics and refactoring inside literate source.

Given an editor pointed at `hick-lsp` for `*.md`, when it makes any request
a mainstream language server answers, then `hick-lsp` forwards it to the
server that owns that block and translates the answer back into document
coordinates. The document is the buffer being edited, so every position in
every reply is a position in the `.md` file — never in the virtual file the
child actually saw.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/the-meta-lsp-forwards-what-the-child-supports.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
