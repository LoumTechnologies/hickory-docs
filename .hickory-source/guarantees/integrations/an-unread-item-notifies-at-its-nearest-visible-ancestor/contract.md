<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Unread Item Notifies At Its Nearest Visible Ancestor

For: Engineers connecting repository issues, reviews and locally installed tools.

Given an unread provider event on a ticket, issue, review, check, or comment,
when its own node is hidden by collapsed ancestors, then its count appears on
the nearest visible ancestor; when expansion makes its node visible, the badge
moves to that node; and expansion alone does not mark the event read.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/integrations/an-unread-item-notifies-at-its-nearest-visible-ancestor.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
