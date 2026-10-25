<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Hover Is Rendered, Not Dumped

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a language server's answer to `textDocument/hover`, when the tooltip is
drawn, then its Markdown is **rendered**: a fenced block becomes a code block
without its backticks or its language word, `---` becomes a rule, a link
becomes its own text, and inline code and emphasis become `<code>`, `<strong>`
and `<em>`. Prose is drawn in the proportional UI font; the fenced signature
stays monospace.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/a-hover-is-rendered-not-dumped.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
