<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Sample Shows Generated Lines Without Storing Them

For: Authors editing executable documents and the files those documents produce.

Given a `<hick:sample path="…" from="…" to="…">` inside the `hick:exec` that
generates that file, when the document is woven, then those lines of the file
appear in the markdown as a fenced block tagged with the file's language,
captioned, and footed with the path and the exact line range shown — and the
`.md` document contains none of those bytes, only the one self-closing
element that names them.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-sample-shows-generated-lines-without-storing-them.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
