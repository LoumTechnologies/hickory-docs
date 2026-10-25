<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Offline edits keep their sync base

For: Host developers embedding the document editor without a native execution service.

Given an initialized local-first workspace, saving edits commits the local
files and their pending publication together. Disconnection leaves those files
editable and saved locally. The original remote revisions and base bytes stay
in a durable sync journal until an explicit sync succeeds. A conflict retains
both the local files and the remote files; it never retries as a blind overwrite.
An edit saved during a slow sync remains pending after that older upload finishes.
Local-save and remote-sync status are separate. Sync is explicit and has no
unbounded polling or background retry loop.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/embedding/offline-edits-keep-their-sync-base.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
