# An Answer In The Agent Pane Has Ribbons

Given a conversation in the agent pane whose turns have been recorded to
a session file — the app's own agent, an ACP agent, or a Claude Code transcript brought
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

## How it reads (amended 2026-09-06, from using it)

The first version showed the file: the XML declaration, the root tags and
every `<hick:usage>` line sat as raw source between the cards, three
elements the harness writes — `reasoning`, `next`, `usage` — were not
registered at all, an answer that only called a tool said "no answer
recorded", nothing wrapped, and the only provenance was the hover-revealed
ribbon. Now: the file's chrome and the record's bookkeeping are folded to
nothing (`sessionChrome`, the `session-meta` kind); the lens wraps; an
answer's reasoning and tool calls are drawn **inside its card**, folded,
the reasoning labelled as such and never mixed with the answer
(`reasoning-is-shown-apart-from-the-answer.md`); and under every answer sit
**source chips** — *read*, *wrote*, *cites* — one per file the turn rested
on, each opening the file at its lines (`linksByAnswer`, `SourceChips`).
The chips and the ribbons are the same links in two forms.

## ACP answers and visibility (2026-10-01)

An answer captured in an unnamed `hick:input` because it quotes hick tags
is still speech: it renders expanded and retains its citations. A markdown
file link ending in `:line` resolves that line, with the suffix removed from
the path. Structured ACP read/search locations and edit locations/diffs
link as adapter-reported evidence; arbitrary shell output is never parsed
into a derivation or an invented file read.

When an answer cites a workspace document and names one of its outputs,
the session endpoint independently weaves that document without executing,
checks the woven output against disk, and reads its byte provenance. Matching
bytes with a source location inside the workspace add a **produces** chip
and a lineage connection from the visible answer to that output. The chip
names the source document; the connection's explanation names the source
and output lines. This is checked **current document evidence**, separate
from the dashed citation and from a record that the turn wrote a file.
Diverged, missing, synthetic-only or unweavable outputs get no such evidence.
The lens refreshes every five seconds while the window is visible; until the
next refresh, its evidence describes the last check. It never rewrites the
session record.

Conversation connections draw for expanded rendered items by default, using
the card's screen bounds. Closed tool/reasoning folds contribute no connection.
The Agent gear offers a persisted **Show lineage for collapsed conversation
items** setting; opting in anchors their connections to the visible summary.
Opening or closing a fold updates the overlay. Conversation visibility is
independent of the document editor's caret preference, and the family's
workspace toggle still applies. Links with no rendered anchor are omitted.
The final answer's source chips collect evidence within its user turn, including
reads and writes recorded before the assistant answer.

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
  `apps/web/src/views/SessionLens.tsx` (`sessionChrome`, `linksByAnswer`);
  `apps/web/src/elements/session/view.tsx` (`SourceChips`, `nested`,
  `INNER`); `apps/web/src/editor/rendered.ts` (a block inside a drawn block
  is not drawn twice); `apps/web/src/lib/lensSources.ts`;
  `apps/web/src/components/ChatDock.tsx` (`sessionPath`, `inLens`);
  `apps/web/src/views/WorkspaceView.tsx` (`lensSources()` in
  `ribbonSources` and `ribbonLinks`).
- Test coverage: `hick-blocks` ("an_element_declares_its_own_links…"),
  `session_elements` tests (blocks, the three families, a Claude Code
  transcript, mentions), `crates/hickory-cli/tests/session_lens.rs` over
  real HTTP, `apps/web/src/views/SessionLens.test.tsx`,
  `apps/web/src/lib/lensSources.test.ts`, `elements/index.test.ts`.

Verification update (2026-10-01): `session_elements::assistant_prose`,
`ContextElement::links`, and `mentions`; `serve/sessions.rs::output_evidence`;
`SessionLens`, `conversationAnchor`, and `RibbonOverlay` establish the paths
above. `session_elements` regressions protect literal quoted tags and structured
ACP locations; real HTTP `tests/session_lens.rs` protects current output matching
and refusal after disk divergence. `SessionLens.test.tsx` and
`conversationLineage.test.ts` protect fold visibility and opt-in behavior.
`just test-conversation-lineage` passes (6 vocabulary tests, 2 HTTP tests,
37 frontend tests, and TypeScript checking); the full web suite passed
(1,841 tests) before the final settings-panel portal change, covered by the
focused regression. The file-length check still fails on four pre-existing
oversized files outside this change; changed oversized files shrink.
Development UI verification against a copied Codex session showed the answer
expanded, a checked produces chip, and a connection to the output in the file
tree. The authenticated adapter was not rerun; the installed app is unchanged. Native adapter reports are not byte-hash
receipts; shell-only reads without structured locations remain unlinked.
