<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Installing A Language Server Is Itself Sandboxed

For: Authors using completion, diagnostics and refactoring inside literate source.

Given `hick lsp install <language>`, when the server is fetched, then the
installer runs **inside the same sandbox a document's cells run in**: it may
write only `.hick-cache/servers/`, its `HOME` is redirected inside that
directory, and it is the one and only place in this product that is granted
network access. Where no sandbox is available, the install is **refused**.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/installing-a-language-server-is-itself-sandboxed.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
