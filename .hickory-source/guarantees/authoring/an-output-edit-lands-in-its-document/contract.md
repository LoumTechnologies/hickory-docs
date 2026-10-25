<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Save a generated edit once and retain it when saving fails

For: Authors editing executable documents and the files those documents produce.

Type a second change while the first generated-file save is pending. The next save must use the acknowledged baseline and retain the newer text.

## What must happen

Supported generated-text edits map to their document source. The frontend computes each diff against the last content acknowledged by the backend; successful saves advance that baseline, failed saves leave it unchanged. Saves are serialized. Incoming output does not replace pending local text, and a discard request rechecks the expected buffer before and after awaiting earlier saves. The filesystem watcher also rechecks contributing source before publication and refuses stale or synthetic edits instead of overwriting unrelated work.

## Limits of the evidence

The focused saver witnesses exercise queued callbacks and byte diffs. They do not establish end-to-end watcher convergence or crash-atomic multi-document publication.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/an-output-edit-lands-in-its-document.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Unify the watcher and HTTP entry points around acknowledged bytes while retaining their distinct concurrency boundaries. Remove historical duplication anecdotes from the current promise.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
