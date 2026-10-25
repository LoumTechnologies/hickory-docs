<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Keep divergent output until an explicit decision names the write

For: Authors editing executable documents and the files those documents produce.

A generated file differs from the document and cannot be mapped back. Leaving the warning must not silently resolve the conflict.

## What must happen

Unreproducible generated output is retained as divergent rather than overwritten. The surface names the write consequence: generated files offer Leave disk unchanged or Regenerate from document; plain buffers offer Overwrite disk with this buffer or Reload from disk. Merge uses the recorded base and current sides under their source guards. Dismissing a notice is not resolution. Kept unrecorded output and an external change under an unsaved buffer remain distinct reasons for divergence.

## Limits of the evidence

Retaining a divergent file is not verification. Per-target guards do not form a filesystem transaction across all products.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/an-output-that-cannot-be-carried-back-is-held.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the common divergence mechanism and replace ambiguous keep-mine/take-theirs labels with the later explicit-consequence wording.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
