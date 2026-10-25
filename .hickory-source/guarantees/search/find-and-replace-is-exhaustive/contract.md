<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Find And Replace Is Exhaustive, And Refuses What It Would Undo

For: Authors finding and replacing project text without missing results.

Given a pattern, when the folder is searched for it, then **every** match is
reported, in path order, or the answer says it stopped and why; and when those
matches are replaced, then exactly the matches that were shown are rewritten,
except in files a document generates, which are refused by name.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/search/find-and-replace-is-exhaustive.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
