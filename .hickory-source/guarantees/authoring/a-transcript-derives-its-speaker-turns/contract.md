<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Transcript Derives Its Speaker Turns, And Cannot Disagree With Itself

For: Authors editing executable documents and the files those documents produce.

Given a `<hick:transcript>` holding the bytes some other tool produced, when
the document is prepared, then its speaker turns are **derived** from those
bytes as `<hick:said>` elements — each carrying who spoke, when, a class per
speaker, and a span into the original — and each is addressable by the selector
grammar that already exists. When the format is not one this knows, then the
raw text survives untouched and no turns are derived.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-transcript-derives-its-speaker-turns.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
