<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Rewritten Link Keeps Its Paragraph Attached

For: Authors editing generated text and reviewers tracing a result to its source.

Given a paragraph of prose in a `.md` document that a transform cuts into
pieces — a `hick:val` substitution, a derived passage — when the document is
woven, then every piece that came through UNCHANGED still carries the source
bytes it came from, narrowed to exactly its own extent; only the bytes a
transform actually replaced are reported as synthetic. A ribbon is drawn from
the sentence in the source to the sentence in the weave, and an edit made on
the woven side still maps back.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/a-rewritten-link-keeps-its-paragraph-attached.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
