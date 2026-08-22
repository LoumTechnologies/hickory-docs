# A Ribbon Crosses Documents, Because Lineage Does

Given a generated file some of whose bytes came from a document other than
the one that generated it — a Slack message that pastes a meeting turn from
a note two `hick:upstream` hops away — when that file is open in the app,
then the overlay draws a ribbon from those bytes to **that** document:
pane-to-pane when its editor is on screen, otherwise to its tab, its row in
the Files tree, or an "open here" port laid out in the divider; and clicking
the ribbon opens that document.

The reason: provenance already names the file each span indexes
(`docs/guarantees/lineage/included-spans-name-their-own-file.md`), and
`hick lineage` prints it. An overlay that dropped every origin outside the
focused document was answering "where did this come from" with "not from
here" for exactly the bytes people most want to trace — the quoted decision,
the number a finding carried, the sentence from the meeting.

Three properties hold it up:

1. **One source per document the outputs reach.** `RibbonOverlay` takes
   `sources`, the focused document first and then every document named by
   the focused document's outputs' provenance, resolved from the path the
   engine reports (absolute) to the path the app knows (tree-relative). A
   document that is open contributes its text and editor; one that is not
   contributes only its path and gets chrome-terminated ribbons.
2. **Terminals are the way there.** A document with no tab gets a port
   (`document:<path>`) in the divider, the same as the focused document gets
   when its own pane is closed; a ribbon ending on a tree row tints that row.
3. **Colour is per fragment per document.** The same byte span in two
   documents is two fragments and two colours.

## Boundary

The overlay still follows the **focused** document's outputs. Focusing the
meeting note shows the meeting's lineage, not the message's ribbons back to
it — provenance is recorded on the generated side. When the target document
was not on screen, the click opens it but the span is not yet selected in
the freshly mounted editor (the same limit the focused document's own
"reopen" path has). Provenance kinds (typed, transcribed, summarized,
executed) are still not drawn differently; see
`docs/specs/freeform/provenance-and-standing.md`.

---

Last LLM verification:
- Date: 2026-08-22
- Reviewer: Claude Fable 5
- Result: verified by driving the served UI (Playwright) against
  `examples/receipts/claude-code`: opening `messages/2026-08-24-eng-reply.txt`
  drew one ribbon titled "Open meetings/2026-08-20-checkout-latency-sync.hick —
  this text came from it" ending on the meeting's inactive tab; clicking it
  navigated to the meeting document. Screenshot:
  `examples/receipts/ribbons-across-documents.png`.
- Evidence: `apps/web/src/shell/Ribbons.tsx` (`sources`, `fragmentIn`,
  document targets carry `path`), `apps/web/src/views/WorkspaceView.tsx`
  (`ribbonSources`, `docIdByPath`, `openDocumentByPath`, document ports).
- Test coverage: `apps/web/src/lib/ribbons.test.ts` covers derivation; the
  multi-source overlay is not unit-tested (it measures DOM) — LLM/manual review.
