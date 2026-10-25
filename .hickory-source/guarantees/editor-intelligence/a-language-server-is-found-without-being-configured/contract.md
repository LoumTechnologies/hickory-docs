<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Language Server Is Found Without Being Configured

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a `hick:file` block whose `path` names a language — `app.py`,
`src/main.rs`, `index.ts` — when `hick-lsp` needs a server for that block,
then it finds one already installed on the machine and uses it, with no
configuration written by anyone. Nothing in the product asks a user which
language server to use, and a repository with no `.hick-lsp.json` gets the
same intelligence as one with a hand-tuned file.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/a-language-server-is-found-without-being-configured.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
