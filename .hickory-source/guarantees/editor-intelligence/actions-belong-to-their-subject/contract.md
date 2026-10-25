<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Keep editor actions attached to the view they affect

For: Authors using completion, diagnostics and refactoring inside literate source.

Move or focus a spatial pane. Its action controls must still target that pane without moving its editor buffer.

## What must happen

Actions retain their owning document/file/view identity and current availability. Current metadata and document/file controls use each spatial pane's existing header. View changes preserve the mounted editor and source identity. A source revision or subject change invalidates stale action context.

## Limits of the evidence

Subject identity and visual placement do not establish that every native action has a registered agent equivalent.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/actions-belong-to-their-subject.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Apply the minimalist-editor amendment while retaining the explicitly preserved visible ruler and action-rail collision behavior.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
