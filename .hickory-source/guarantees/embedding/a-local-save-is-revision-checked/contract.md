<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A local save is revision checked

For: Host developers embedding the document editor without a native execution service.

Given a file read from browser-local storage with an opaque revision token,
when another tab changes it before a save or delete,
then the stale operation fails and preserves the newer stored bytes and the
editor's unsaved draft. Success is reported only after the IndexedDB transaction
commits. Quota/transaction failures do not mark the draft saved. Memory storage
identifies itself as memory only. Workspace imports either create every file or create none; conflicts never
partly replace a workspace. Workspace exports capture a consistent set
of source, binary assets and evidence without reclassifying their provenance.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/embedding/a-local-save-is-revision-checked.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
