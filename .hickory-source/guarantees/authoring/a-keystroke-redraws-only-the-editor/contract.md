<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Keystroke Redraws Only The Editor

For: Authors editing executable documents and the files those documents produce.

Given a document open in the workspace beside any number of other panes, when
a character is typed into it, then the only React render that keystroke causes
is inside the editor pane that received it — the workspace chrome (the other
panes, the folder tree, the rulers, the rails, the ribbon overlay, the status
bar's other fields) is not re-rendered, and nothing that was already on screen
is re-measured against the DOM.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-keystroke-redraws-only-the-editor.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
