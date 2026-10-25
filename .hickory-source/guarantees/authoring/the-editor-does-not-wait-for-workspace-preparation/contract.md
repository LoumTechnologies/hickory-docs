<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Editor Does Not Wait For Workspace Preparation

For: Authors editing executable documents and the files those documents produce.

Given a desktop window opens a workspace, when its watcher has been installed
and startup configuration validated, then the window can load before the initial
weave finishes. Settings, folder identity, layout and draft recovery remain
available. An unsaved note can be edited and its recovery draft persisted while
generated files are being prepared.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
