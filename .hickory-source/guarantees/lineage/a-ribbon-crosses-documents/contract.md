<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Trace a ribbon to the visible part of its source

For: Authors editing generated text and reviewers tracing a result to its source.

Narrow or horizontally scroll the Files pane while a ribbon points into it. The endpoint must remain on the visible row or fall back to the divider.

## What must happen

Cross-document lineage, context and declared links retain their recorded source identity and use the appropriate visible editor, Files row or divider endpoint. Files-row endpoints are clipped on both axes by every scrolling or clipping ancestor, excluding borders and scrollbar space. Fully clipped rows use the existing divider fallback. Tree scrolling schedules overlay updates. This presentation rule changes neither provenance semantics nor citation support.

## Limits of the evidence

The recorded geometry review assumes the shell's ordinary untransformed layout. Rotated/scaled custom hosts and semantic truth of citations are outside that witness.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/a-ribbon-crosses-documents.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Fold the October 8 visible-row correction into the current endpoint contract. Retire the rejected Input-marker presentation proposal as history.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
