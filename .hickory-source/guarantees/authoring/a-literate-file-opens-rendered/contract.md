<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Literate File Opens Rendered, And Its Source Is One Click Away

For: Authors editing executable documents and the files those documents produce.

Given a `.md` document, when it is opened in the app, then every exec cell
and every diagram shows its RESULT — the diagram drawn, the cell's commands
and last output — rather than its tags; and when the block's rail icon (or its
`source` control) is clicked, then that block shows its raw source, editable
as ordinary text, and clicking again returns it to the result.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-literate-file-opens-rendered.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
