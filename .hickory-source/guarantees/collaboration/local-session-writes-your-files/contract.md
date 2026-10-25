<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Persist local document edits in ordinary files

For: Authors switching between the desktop editor, their other editor, and Git.

After a desktop edit is persisted, reading the Markdown file from another editor or git diff must show that edit without an export step.

## What must happen

Given a local document room, persisted source edits land in the Markdown file on that machine. The room uses the shared hickory-collab machinery and file-backed store. Document writes use a sibling temporary file and rename; closing the last socket requests a final persist. The CRDT sidecar supports room reconstruction and is not a replacement for the source file. The local service boundary and optional authenticated peer transport have separate contracts.


## Limits of the evidence

A local document store does not imply that optional fleet transport is absent. Socket-close ordering, crash behavior and frontend acceptance require their own witnesses; this rewrite does not rerun the HTTP suite.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/local-session-writes-your-files.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Remove the global no-sharing/no-relay claim from the local-persistence contract. Optional peer transport remains governed by its existing grant and disclosure guarantees.

Implementation review: partially verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
