<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Notebook Asks The Same Questions An Editor Does

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a document open in the notebook — the editor the desktop app hosts —
when the editor asks a language question (hover, definition, references,
completion) or the document changes, then the answer comes from the same
`hick-lsp` that serves Zed, VS Code, Helix, and Neovim, delegating to the same
child language servers, and arrives in **document coordinates**.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/the-notebook-asks-the-same-questions-an-editor-does.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
