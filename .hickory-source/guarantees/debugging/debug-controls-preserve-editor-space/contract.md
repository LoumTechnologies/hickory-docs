<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Choose a launch without replacing the editor being read

For: Authors debugging generated code while navigating its document source.

Open the launch picker while reading generated code. Choosing a target should not replace the editor or silently switch the code being debugged.

## What must happen

Debug controls preserve the mounted editor and its available reading space. Launch profiles use the current floating picker and exact displayed-source selection. Source identity is rechecked by the host; a stale or unavailable mapping remains unavailable. Ordinary language-service scope is independent of the selected debugger launch.

## Limits of the evidence

A controller test does not establish native visual layout across every viewport.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/debug-controls-preserve-editor-space.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Use the launch-profile continuation instead of the earlier global version-menu and replacement-editor wording.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
