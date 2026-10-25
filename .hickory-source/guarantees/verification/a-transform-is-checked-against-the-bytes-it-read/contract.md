<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Transform Is Checked Against The Bytes It Read, Wherever They Live

For: Reviewers deciding what an output, recording or prediction actually establishes.

Given a `<hick:transform select="…" instruct="…" from="…">`, when `hick test`
checks it or `hick refresh` rewrites it, then the fingerprint is taken over
the **same bytes `hick:paste` would find at weave** — including fragments the
document reaches through `hick:upstream` or `hick:include` — so a passage
summarizing a meeting turn in another document goes stale the moment that
turn changes. When `hick refresh` writes a passage, then it stamps `from=`
**and** the `provider=` and `model=` that wrote it, by the tag's own span, so
a document that binds another prefix (`<slack:transform>`) is restamped too.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/a-transform-is-checked-against-the-bytes-it-read.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
