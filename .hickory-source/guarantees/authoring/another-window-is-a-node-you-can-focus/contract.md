<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Another Window Is A Node You Can Focus

For: Authors editing executable documents and the files those documents produce.

Given two live Hickory Docs desktop processes with different folders, when one
workspace tree shows the other's window node and that node is activated, then
the other native window is unminimized, raised, and focused; stale registrations
are not shown; and a browser-only `hick up` process does not pretend it has a
window to focus.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/another-window-is-a-node-you-can-focus.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
