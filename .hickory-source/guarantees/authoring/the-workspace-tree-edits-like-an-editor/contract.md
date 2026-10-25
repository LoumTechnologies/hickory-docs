<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Workspace Tree Is An Editor Buffer

For: Authors editing executable documents and the files those documents produce.

Given an open folder, when the Files pane is shown, then the filesystem is
written into an ordinary CodeMirror buffer: one entry per line, a trailing
slash distinguishes a folder, and exactly two leading spaces per level place
an entry inside the folder above it. The bytes are the tree. They are not
labels laid over buttons and there is no separate edit mode.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-workspace-tree-edits-like-an-editor.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
