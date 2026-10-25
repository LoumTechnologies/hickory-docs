<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Pasting A URL Over Words Makes A Link

For: Authors editing executable documents and the files those documents produce.

Given a selection of prose in a document and a URL on the clipboard, when the
paste happens then the selected words become the label of a markdown link
pointing at the pasted address — `[the spec](https://example.com/spec)` — and
the caret lands after it. With nothing selected, or with something on the
clipboard that is not a URL, the paste is the ordinary paste it has always
been.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/pasting-a-url-over-words-makes-a-link.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
