<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The File Menu Preserves The Originating Window

For: Authors editing executable documents and the files those documents produce.

Given a Hickory Docs window with no folder open, File → Open Folder or a
recent folder attaches the selected folder to that same window. The native
window and editor origin stay in place; existing tabs, unsaved unnamed notes,
recovery drafts, and explicitly opened files stay available. Files appears
beside the editors. This also applies to a window launched on a single file.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/file-menu-preserves-the-originating-window.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
