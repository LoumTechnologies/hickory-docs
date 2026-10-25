<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Keep an existing artifact when no recording can construct it

For: Reviewers deciding what an output, recording or prediction actually establishes.

Open a captured project whose transcript is absent locally. Weaving must not replace a retained output with a never-run marker.

## What must happen

When a required cell has no recording, weaving preserves existing outputs it cannot derive, including the woven report, and reports that preservation. A stale recording is different: weaving may display its last recorded bytes with stale status and write that rendering. Only an absent baseline is unrecorded. Explicit execution can refresh the source outputs; an unrecorded weave must not silently destroy retained evidence.

## Limits of the evidence

Retaining an artifact does not verify its current inputs. A newly absent output cannot be recovered from a recording that does not exist.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/a-weave-without-a-recording-keeps-the-artifact.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Apply the September 3 stale-versus-unrecorded amendment to the September 2 preservation rule. Keep missing-baseline and changed-input outcomes separate.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
