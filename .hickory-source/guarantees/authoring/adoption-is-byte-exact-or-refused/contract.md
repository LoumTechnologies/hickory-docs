<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Adopt a plain file without changing its bytes

For: Authors editing executable documents and the files those documents produce.

Make a plain source file literate. The new document must regenerate the exact original file before adoption is accepted.

## What must happen

Supported plain-text adoption verifies the constructed output against the original before keeping the change. Failure restores the affected source/output state. Appending refuses an already-owned path and preserves the document's other generated products; its own woven reading may change as the authored document changes. The current adoption path refuses unsupported binary/document inputs and source syntax it cannot represent exactly.

## Limits of the evidence

Declared-project raw/hex entries can construct inputs that the plain-file adoption command does not support; those are separate interfaces.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/adoption-is-byte-exact-or-refused.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Fold the bare-document amendment into the preservation rule. Do not promise that the woven reading stays unchanged when the maintained document itself changes.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
