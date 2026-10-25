<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Search Works Offline; Only An Explicit Install Touches The Network

For: Authors finding and replacing project text without missing results.

Given a machine with no network access, when `hick search "query"` (or the
app's search panel, or the MCP `search` tool) runs, then it answers from a
locally built index with lexical (BM25) ranking, builds and caches that index
under `.hick-cache/search/`, and never attempts a network request.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/search/search-is-offline-by-default.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
