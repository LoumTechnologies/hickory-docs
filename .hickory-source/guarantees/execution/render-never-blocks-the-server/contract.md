<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Rendering a Document Never Blocks the Server

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given any number of documents being rendered at once, when a client calls
`GET /api/docs/:id/render`, then the weave and the project-checkout seed run
on the blocking pool — never on an async runtime worker — so unrelated
requests (`GET /api/health` among them) keep answering promptly; and when
nothing the weave depends on has changed, the render is served from a
bounded in-process cache instead of being recomputed.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/render-never-blocks-the-server.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
