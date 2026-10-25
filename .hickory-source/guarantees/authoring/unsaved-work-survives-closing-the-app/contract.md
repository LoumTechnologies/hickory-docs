<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Close or recover a document without losing the wrong draft

For: Authors editing executable documents and the files those documents produce.

Close a changed note, choose Discard, then reopen the workspace. The discarded draft must stay discarded. A crash instead recovers the last retained draft as unsaved work.

## What must happen

A buffer remains dirty until its actual file write is acknowledged. Tab close offers Save, Discard and Cancel; window close offers Save all, Close without saving and Cancel. Explicit discard removes the corresponding recovery draft and suppresses late writes from that discarded buffer. Save failure, cancellation or changed source identity keeps the affected view open. Each unnamed note has independent recovery outside the project. Named-file recovery follows its setting, which defaults off. Recovery does not create a project file or acknowledge a save. A recovered draft is compared with its recorded base and current file before restoration or merge. An absent editor view must not be read as an empty buffer.

## Limits of the evidence

Periodic recovery can lose input after its last successful flush. Frontend tests mock persistence and do not certify native window destruction or crash durability. Keep the original merge, path-renaming and startup-note clauses as detailed requirements.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Fold the explicit-discard, in-flight-write and dead-view corrections into one current lifecycle rule. Keep crash recovery distinct from an intentional close without saving.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
