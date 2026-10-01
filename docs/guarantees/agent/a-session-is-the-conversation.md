# A Session File Is The Conversation, And Opens As One

Given a conversation in the chat dock, when a turn runs, then it is appended
to ONE `.md` session file per conversation — a root turn starts the file, a child
turn appends to its parent's — and its `<hick:user>` records the turn id, the
parent turn id, and the provider and model it ran on; when the app restarts,
then the dock's turn tree for a document is rebuilt from those files; and
when a session file is opened as a document, then it is drawn — in the
editor, over its own bytes — as the conversation it records: each top-level
turn a speech bubble, yours on the right and right-aligned, the agent's on
the left (the sides Apple Messages uses), the agent's work (tool calls, scripts, observations,
results, reasoning) folded to its opening and closing tag lines; and every
line of the file is still on screen with its line number, its ribbons, and
its tags — the bubbles are formatting, not a second surface.

The reason: the file IS the record, so the file and the dock must not tell
two stories. Before this, each turn wrote its own unlinked file, the tree
lived only in memory (gone on restart), and a session opened in the app was
XML with chips — the thing you were shown while it happened and the thing
you could find afterwards were different things.

Four properties hold it up:

1. **One conversation, one file.** `TurnRecord::session` names it;
   `AgentConfig::session_path` appends (`HickSessionLog::append_or_create_for`);
   the root element names the document (`doc=`).
2. **The tree is in the file.** `<hick:user turn=… parent=… provider=… model=…>`
   (`SessionEvent::UserTurn`); `session_view` reads it back; `AgentHub::hydrate`
   rebuilds the dock from `sessions/` for a document the hub has no turns for.
3. **One surface.** An opened session is the editor, not a viewer: line
   decorations in `editor/wysiwyg.ts` (`cm-bubble-you` / `cm-bubble-agent`,
   first/last) make top-level turns bubbles, nested elements keep a plain
   frame inside them, and `editor/folding.ts` (`WORK_BLOCKS`,
   `sessionWorkFolds`) folds the work once when the file first has content —
   keeping each work element's closing tag line visible, so the whole element
   reads even folded. There is no Chat/Source toggle: the source IS the chat.
   The dock draws the live turn tree with `SessionTurns`/`TurnCard` over
   `GET /api/sessions/view?path=`, the same shape, the same bubble language.
4. **Zoom out and move.** The dock's Tree view lays turns out as a DAG (the
   git pane's lane layout), highlights the current branch, and a click makes
   a node the tip; `/rewind [N]`, `/tree`, `/new`, `/help` do the same from
   the composer.

## Boundary

Reasoning appears only when the provider streams it; a turn that shows none
had none exposed. CLI runs (`hick agent`) still write one file per run — they
are not turns of a dock conversation. Hydrated turns report usage summed from
the file's `<hick:usage turn=…>` rows and no per-turn cost when the model has
no price.

---

Last LLM verification:
- Date: 2026-08-22
- Reviewer: Claude Fable 5
- Result: verified by driving the served UI: a dock turn through OpenRouter
  wrote `sessions/<ts>-<slug>.hick` with `doc=`, `turn=`, `provider=`,
  `model=` on the user element; `/rewind` then a second turn started a
  second root; the Tree view showed both with the tip marked
  (`examples/receipts/chat-tree.png`); the turn's "session" button opened
  the file as a conversation with a folded reasoning step and the answer
  (`examples/receipts/session-chat.png`).
- Evidence: `crates/hickory-agent/src/session_view.rs`; `SessionEvent::UserTurn`,
  `create_for`/`append_or_create_for`, `write_reasoning` in
  `crates/hickory-agent/src/session.rs`; `AgentConfig::{session_path,turn_id,parent_turn_id}`
  in `react_loop.rs`; `AgentHub::hydrate`, `TurnIdentity` in
  `crates/hickory-cli/src/serve/agent.rs`; `session_view` in `serve/api.rs`;
  `apps/web/src/components/{SessionTurns,ChatTree,ChatDock}.tsx`;
  `apps/web/src/views/workspaceTabs.tsx` (Chat/Source).
- Tests: `session_view::tests`, `ChatDock.test.tsx` (slash commands, rewind,
  path), `session.rs::tests`.
- 2026-08-23 (later): problem markers and a cleaner frame. `hick-lsp` now
  lints a `hick:session` (`crates/hick-lsp/src/session_lint.rs`, published
  from `process_document` under the document's own URI beside the children's
  diagnostics): an unanswered turn, a tool call with no result, a `parent=`
  naming no turn (warnings), two turns sharing an id (error); a refused tool,
  a non-zero exit, a compaction, and an import's broken close tag are
  *information* — marked (`.cm-lsp-info`, a dotted line) and listed, never
  counted. `crates/hickory-cli/tests/lsp_desktop.rs::a_sessions_problems_reach_the_window_as_diagnostics`
  drives them through the app's own WebSocket bridge. In the editor the turn
  chip now LEADS the opening line (the bubble's caption, with the clock time
  from `at=` and the assistant's model), the session root lines are framed
  like a `hick:doc` envelope, and tag chrome steps back inside a bubble.
- 2026-08-23: the Chat/Source toggle is gone. A session opens in the editor
  as bubbles (`session.test.tsx` asserts the sides, the first/last lines on
  the tag lines, and that the action's script opens folded while what was
  said does not; `folding.test.ts` asserts work folds keep the closing tag
  line). Verified by driving the served UI on the every-turn-chip fixture:
  line numbers 1–46 count straight through, the `you` bubble right, agent
  bubbles left with tails, tool/action/observation folded to `…` with their
  close tags visible, no horizontal overflow.
- 2026-08-22: the turn cards became message bubbles (`.chat-bubble`, tail on
  the speaker's side, yours right and the agent's left; the composer's draft
  is a bubble too). The earlier layout was a two-column grid with a role
  label in the first column, and the "show work" block auto-placed into that
  3.6rem column, wrapping every few characters — the session fixture
  `sessions/20260820-090000-every-turn-chip.hick` showed it. Verified by
  driving the served UI: the fixture's 11 steps open inside the agent's
  bubble at full width.


Verified 2026-10-01 (Markdown document extension): `hickory_agent::session_file_path` now writes `.md`; `conversations_for`
reads Markdown sessions as well as older session records.
`session::tests::new_sessions_use_the_markdown_document_extension` checks the
conventional path, and the conversation/context fixtures now use `.md`.
All 90 agent unit tests pass.
