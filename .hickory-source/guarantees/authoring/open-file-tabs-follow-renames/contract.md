<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Open File Tabs Follow Renames

For: Authors editing executable documents and the files those documents produce.

Given a plain file open in an editor pane, when it is renamed from Hickory's
Files tree, then its existing tab keeps its pane, position, and editor state
while its path and displayed filename change to the new name. It is one file
moving, never a close followed by a second editor opening.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/open-file-tabs-follow-renames.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
