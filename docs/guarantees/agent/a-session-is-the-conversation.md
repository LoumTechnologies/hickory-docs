# A Session File Is The Conversation, And Opens As One

Given a conversation in the chat dock, when a turn runs, then it is appended
to ONE session file per conversation — a root turn starts the file, a child
turn appends to its parent's — and its `<hick:user>` records the turn id, the
parent turn id, and the provider and model it ran on; when the app restarts,
then the dock's turn tree for a document is rebuilt from those files; and
when a session file is opened as a document, then it is drawn as the
conversation it records — the same turn cards the dock draws, with the
agent's reasoning folded, its scripts, tool calls, observations, files read
and lines written under "show work", and its answer — with the source one
click away.

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
3. **One renderer.** `SessionTurns`/`TurnCard` draw both the live dock and an
   opened session (`SessionDocView`, behind a Chat/Source toggle on the tab);
   `GET /api/sessions/view?path=` is the shape both read.
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
