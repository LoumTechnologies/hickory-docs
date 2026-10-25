<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A comparison keeps current code editable

For: Authors comparing current code with another revision while continuing to edit.

Given a literate reading or persistent document, comparing it with a Git
revision leaves the current code in the main editor's actual buffer. Added lines
update as it is edited. Removed lines are shown by a minus marker in the gutter. Clicking it opens a
floating previous-version preview with an explicit restore action on editable
comparisons. No historical rows take space in the text. Removed bytes enter the
current buffer only through explicit restoration. They do not affect saved text or language-server coordinates.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editing/a-comparison-keeps-current-code-editable.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
