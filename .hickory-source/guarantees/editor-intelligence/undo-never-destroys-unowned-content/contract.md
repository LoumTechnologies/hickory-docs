<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Undo Only Undoes What This Client Typed

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a document open in the collaborative editor, when the user presses undo
repeatedly — past the start of their own edits — then no content arrives at or
disappears from the document that this client did not type: not the initial
sync, not a collaborator's edits, not the mock-mode seed. Undo still fully
undoes this client's own typing.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/undo-never-destroys-unowned-content.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
