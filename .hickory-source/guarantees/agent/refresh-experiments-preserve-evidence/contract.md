<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Local refresh experiments preserve historical request evidence

For: Authors using an agent to change documents while retaining review control.

Given an explicitly queued update, the Python refresh policy supports appended
replacement passages, exact prior-read projection with a change notice, and
notification followed by a bounded reread. Scheduling remains a separate input.
Projection assembles a copy; original request bundles and caller history remain
unchanged. It replaces only exact role-matching plain text messages and leaves
tool-call envelopes intact. Source updates remain user notices; role/scope is
reported rather than promoted. Notices invalidate earlier dependent plans and
summaries without claiming to have corrected their reasoning.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/refresh-experiments-preserve-evidence.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
