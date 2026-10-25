<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A failed action keeps its editor

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a loaded document or generated file, when a Run, Test, render, or
save request fails, then its editable work remains reachable and a local notice
names the failure and offers the corresponding retry. Only a failed initial
load can replace the document editor. A failed template listing offers retry
without leaving New Project; unsupported SDK installation remains a link.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-failed-action-keeps-its-editor.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
