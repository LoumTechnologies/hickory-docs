<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Ingest Keeps The Original Bytes, Calls No Model, And Never Deletes Your File

For: Authors editing executable documents and the files those documents produce.

Given a file in the notes folder's inbox, when it is ingested, then a `.md`
note is written holding the file's bytes verbatim, its attendees derived from
who actually spoke, and its summary passages **empty and stale**; and the source
file is **moved** into `inbox/ingested/`, never deleted. When the same bytes are
ingested again, then no second note is written.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/ingest-keeps-the-original-bytes.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
