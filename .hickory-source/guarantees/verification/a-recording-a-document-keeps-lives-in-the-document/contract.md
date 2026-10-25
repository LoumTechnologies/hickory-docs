<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Recording A Document Keeps Lives In The Document

For: Reviewers deciding what an output, recording or prediction actually establishes.

Given a document whose cells are recorded, when `hick ingest --from
recording <doc>` runs, then each recorded cell's output is written into the
document as `<hick:ingested key="…" sha256="…" at="…">` inside the cell,
verbatim; the weave and `hick run` look there first and in the cache
second, match it by the same key the cache uses, and report it *stale* the
same way when the cell's inputs move; `hick run` brings a kept recording
forward when it re-executes the cell, and never adds one the document did
not keep; a clone with no cache weaves the same bytes; and nothing is
written unless the document with its recordings inside weaves, against an
empty cache, every byte the cache-backed weave did. `.hick-cache/` is a
cache again, whole, in `.gitignore`.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/a-recording-a-document-keeps-lives-in-the-document.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
