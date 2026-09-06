# An Answer In The Agent Pane Has Ribbons

Given a conversation in the agent pane whose turns have been recorded to
a session file — the app's own agent, or a Claude Code transcript brought
in with `hick ingest --from claude-code` — when the pane draws it, then it
draws the session document itself: line numbers, each element as the card
it always was, read-only; and from those lines the ribbon overlay draws
three families of connection, kept apart the way `three-provenances.md`
keeps them: **context** from every `read` (or `Read` tool call) to the
file and lines the model was shown; **lineage** from every `wrote` (or
`Edit`/`Write` call) to the lines the turn left in a document; and
**declared**, dashed, from an assistant's prose to every file it links or
names as `path:line`. Each lands on the far file's tab, tree row, or port,
and a click opens it there. The status bar's family toggles apply.

Until this, the pane rendered its own cards from its own API and the
overlay could anchor only on editors; an answer could cite nothing, and
what the model had been shown was visible only by opening the session file
by hand. The record was there; the view was not a view of it.

Three properties hold it up, and each is the general thing rather than a
chat special case (`docs/specs/freeform/the-minimal-core.md`):

1. **An element declares its links.** `hick_blocks::Element::links` sits
   beside `render` and `act`; `Registry::links` walks a document as
   `blocks` does. The session elements (`hick_literate::session_elements`)
   declare theirs from the record — `read`, `wrote`, tool calls with a file
   argument in either hick's or Claude Code's names, and the prose's
   markdown links and `path:line` mentions — never from the model's account
   of itself. `GET /api/sessions/view` answers the blocks, the links with
   the lines their spans cover, and the source.
2. **The pane is a lens.** `SessionLens` is a read-only editor over the
   session file with the same `renderedBlocks` machinery and the same
   element views (`elements/session`) every document uses; a turn still
   running, or one that failed before it was recorded, stays a live card
   beneath it until the file has it.
3. **The overlay draws lines, full stop.** The lens registers its editor,
   text and links (`lib/lensSources.ts`); the workspace hands them to the
   overlay as one more source with its links, and nothing in the overlay
   knows what a chat is.

## Boundary

The far end lands on chrome (tab, tree row, port), not on the target's
lines when it is open beside the pane — the same open edge every context
and declared link has. Per-turn branch navigation lives in a strip under
the lens rather than on each card; `/rewind`, `/tree` and `/new` are
unchanged. A `read` that hick's harness did not write — an older session —
has no context link; a Claude Code `Read` without offset and limit links
the whole file.

---

Last LLM verification:
- Date: 2026-09-05
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hick-blocks/src/lib.rs` (`Link`, `Family`,
  `Element::links`, `Registry::links`);
  `crates/hick-literate/src/session_elements.rs` (the elements, `mentions`,
  `session_blocks`, `session_links`); `crates/hickory-cli/src/serve/api.rs`
  `session_view`; `apps/web/src/elements/session/view.tsx`;
  `apps/web/src/views/SessionLens.tsx`; `apps/web/src/lib/lensSources.ts`;
  `apps/web/src/components/ChatDock.tsx` (`sessionPath`, `inLens`);
  `apps/web/src/views/WorkspaceView.tsx` (`lensSources()` in
  `ribbonSources` and `ribbonLinks`).
- Test coverage: `hick-blocks` ("an_element_declares_its_own_links…"),
  `session_elements` tests (blocks, the three families, a Claude Code
  transcript, mentions), `crates/hickory-cli/tests/session_lens.rs` over
  real HTTP, `apps/web/src/views/SessionLens.test.tsx`,
  `apps/web/src/lib/lensSources.test.ts`, `elements/index.test.ts`.
