<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Select model context without saving open buffers

For: Authors using an agent to change documents while retaining review control.

Keep unsaved text open but leave it out of Next context projection. Its edit identity may be captured, but opening the tab alone must not send its full text to the model.

## What must happen

Open buffers can supply exact edit snapshots without becoming model-visible text. The current workspace projection explicitly selects expanded or pinned text for the next capture. Unselected open files contribute identity and edit snapshots, not their complete contents. Captured unsaved bytes remain unsaved. Named buffers retain file identity; identifiable untitled buffers use editor-only addresses rather than invented paths. Historical receipts describe what was actually delivered and do not certify today's source or claim support.

## Limits of the evidence

Opening an editor is not evidence of provider delivery. Opaque ACP filesystem tools retain their independent access. The historical HTTP witnesses do not establish the current Python projection.

Current delivery authority: [selected reading projection](../expanded-reading-projection/contract.md).

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/the-agent-sees-the-open-editors.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: The October 18 selected-reading requirement supersedes the earlier send-all-open-editors contract. Keep legacy delivery behavior as historical evidence and make expanded-reading-projection the current delivery authority.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
