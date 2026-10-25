<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# `.md` Documents Merge Through Hick, And A Clone That Cannot Is Told

For: Engineers using local sessions or pairing their own machines.

Given a repository set up with `hick init`, when git merges a `.md`
document, then it runs hick's merge driver rather than its own line merge —
and when a clone has the routing but not the driver definition, then that is
reported at project open and by `hick test`, because git otherwise falls back
silently.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/hick-documents-merge-through-hick.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
