<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A `hick:file` Is A Generated File However Deeply It Is Nested

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a `<hick:file path="…">` anywhere in a document — at the top level, or
inside a `hick:exec`, a `hick:ingested`, or anything else — when the document
is read by the language server, the debugger, or a `<hick:capture>`, then that
block is one of the document's generated files: it has an embedded language
server, a breakpoint gutter, and a place in the debugger's weave. The rule is
one rule, stated once: **the file blocks the engine's own weave writes**
(`all_tags`), never a subset of them.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/a-file-block-is-a-file-however-deeply-it-is-nested.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
