<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Saved means the displayed bytes reached the file

For: Authors editing executable documents and the files those documents produce.

Given an edited named document, ordinary file, generated-file buffer, or
literate view, when its save succeeds, then only the acknowledged text becomes
clean. An earlier save response cannot mark later typing saved. Rendering
failure does not invalidate acknowledged document persistence. A failed or
conflicted write stays dirty and closing waits for a successful Save or an
explicit discard/retention choice.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
