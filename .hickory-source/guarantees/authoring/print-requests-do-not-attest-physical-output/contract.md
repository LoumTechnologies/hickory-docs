<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Shared print requests do not attest physical output

For: Authors editing executable documents and the files those documents produce.

Given a focused printable buffer, when the human or agent requests Print, both
invoke the same full-buffer print callback. The title comes from the active tab
in the focused pane, rather than another pane's active tab. An absent editor is
unavailable; a platform callback error remains an error. A returned callback does
not establish that a dialog appeared, was confirmed rather than cancelled, or
produced physical output. Printer selection is browser/OS-owned, beyond what the
page's print API reports; the registered command states this limitation before
invocation and returns its actual callback result afterward.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/print-requests-do-not-attest-physical-output.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
