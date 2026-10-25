<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Ingested Block Is Marked In The Document View Too

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a `<hick:ingested>` block's content sitting in the Document view —
plain, editable text, since the owning exec cell's card deliberately stops
rendering right before it — when the view draws it, then it carries the
same `cm-prov-ingested` dashed-underline treatment the Output pane's
per-character lineage already gives ingested bytes there, not zero visual
marking.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/an-ingested-block-is-marked-in-the-document-view-too.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
