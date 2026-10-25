<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Spatial Views Preserve The Editor

For: Authors arranging document windows while preserving their editing position.

Given a workspace, Appearance settings independently enable the left and right
drawers; left defaults on and right defaults off. An enabled drawer appears only
when it has content: a populated Files browser, a relationship to the focused
view, a pending review, or an explicitly pinned view. Unrelated open tabs alone
do not create drawer content. Empty drawers have no handle and reserve no space.
Populated drawers start collapsed and reserve no space. Opening overlays the
editor below its tabs; pinning reserves space beside it. Overlay dismissal by
Escape, close, or a click outside preserves editor buffers and selections.
Drawer pins persist per folder; transient open state does not.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/shell/spatial-views-preserve-the-editor.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
