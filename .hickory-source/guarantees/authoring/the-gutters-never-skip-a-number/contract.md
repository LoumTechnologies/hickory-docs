<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Gutters Never Skip A Number

For: Authors editing executable documents and the files those documents produce.

Given a document open in the app, when it contains any element the editor
annotates — an exec cell, a generated file, a copy/cut fragment, a container,
a conditional, a feature, a session turn, a diagram — then every screen row in
the editor is either one document line or a rendered block standing in for a
contiguous run of lines, and no decoration ever adds a row that no line
accounts for.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-gutters-never-skip-a-number.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
