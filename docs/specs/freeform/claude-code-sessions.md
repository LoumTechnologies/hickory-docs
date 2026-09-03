# Claude Code transcripts as session documents

*Status: shipped 2026-08-23 as `hick ingest --from claude-code`. This note records
what a Claude Code transcript contains, what our `hick:session` format
carries, where the two differ, and what was decided about each difference.
Guarantee: `docs/guarantees/agent/a-claude-code-transcript-imports-as-a-session.md`.*

## What a Claude Code transcript holds

`~/.claude/projects/<project>/<session-id>.jsonl`, one JSON object per line.
A census of 37 transcripts of this repository (66,000 records) and a sample
across other projects found these record types:

| type | what it is | count (this repo) |
|---|---|---|
| `user` | a prompt you typed, OR a `tool_result` block, OR a meta note (`isMeta`), OR a compaction summary (`isCompactSummary`) | 20,787 |
| `assistant` | one content block per record (`text`, `thinking`+`signature`, `tool_use`), records of one reply share `message.id`; carries `model`, `usage`, `stop_reason` | 32,918 |
| `attachment` | things the harness put in front of the model: `hook_success` (hook stdout), `file`, `edited_text_file`, `queued_command`, `date_change`, `total_tokens_reminder` (2,121 of them), `task_reminder`, tool/skill listings | 35,279 |
| `system` | `compact_boundary` (with pre/post token counts and what was preserved), `turn_duration`, `local_command`, `away_summary` | 863 |
| `queue-operation` | a prompt you typed while a turn was running | 1,496 |
| `ai-title`, `last-prompt`, `mode`, `permission-mode` | harness state, rewritten constantly | ~15,000 |
| `file-history-snapshot/-delta`, `frame-link`, `agent-name`, `worktree-state`, `bridge-session`, `atis-latch`, `relocated` | bookkeeping | ~1,500 |

Every conversational record carries `uuid`, `parentUuid` (the tree — a
compaction boundary has `logicalParentUuid` instead), `timestamp`, `cwd`,
`gitBranch`, `version`, `sessionId`. Subagents live in
`<session-id>/subagents/agent-*.jsonl`.

## What our session format holds

`hick:session` (root, `start=`, `doc=`) → `hick:user` (`turn=`, `parent=`,
`provider=`, `model=`), `hick:assistant` holding prose, `hick:reasoning`,
`hick:tool` (`hick:arg`, `hick:input`), `hick:action`; `hick:tool-result`,
`hick:observation`, `hick:usage`, `hick:read`, `hick:wrote`. Four elements are
captured verbatim by the parser: `input`, `tool-result`, `reasoning`, and —
added with this work — `context`.

## Where they differ, and what was decided

| Claude Code has | We had | Decision |
|---|---|---|
| a per-record timestamp | `start=` only | `at=` on user, assistant, tool-result, context. Shown nowhere yet; carried so nothing is lost. |
| `tool_use.id` ↔ `tool_result.tool_use_id` | results named only by tool name | `call=` on both sides. The viewer still joins by order; the id is there for anything stricter. |
| `thinking` + `signature` | `hick:reasoning` | reasoning kept verbatim; the signature (an opaque blob for replay on one provider) dropped and counted. |
| what the harness showed the model (attachments, compaction, meta prompts) | `hick:read` for files the built-in agent's tools showed | **new `hick:context kind= at=`**, verbatim. Shown folded in the dock ("show work") and framed/folded in the editor like other work. The built-in agent does not write it; `read` remains the precise form. |
| images in prompts and results | nothing | a placeholder naming type and size; bytes dropped and counted. A session is text; an asset store for session images is a separate decision. |
| session id, cwd, branch, harness version, AI title | `doc=` | root attributes `source= session= cwd= branch= harness-version= title=`. |
| `stop_reason`, `requestId`, `effort`, structured `toolUseResult` | nothing | dropped, not counted — they describe the API call, not the conversation. `toolUseResult` duplicates the text the model saw. |
| reminders (`total_tokens_reminder`, `task_reminder`) | — | dropped and counted: hundreds per session, all one sentence. |
| subagent transcripts | — | not imported with the parent. Each is a transcript of its own shape and can be imported on its own; linking them is future work. |

## What the import teaches about the format

1. **The no-escaping invariant bites transcripts of work on hick.** Seven of
   this repo's 37 transcripts would not import until the three rules in the
   guarantee existed (wrap quoting prose in `hick:input`; JSON for a quoting
   tool input; break a literal close tag with a space). Transcripts of other
   projects never hit any of them. A future parser could offer a heredoc-style
   terminator for verbatim elements (`<hick:input until="EOF">…</hick:input EOF>`)
   and remove the third rule; it was not worth a grammar change today.
2. **`<!--` is structure too**, and an unclosed one swallows the document. The
   "does this text need a verbatim wrapper" test must include it.
3. **Usage adds up per turn**, and a Claude Code turn is many replies; the
   per-reply `hick:usage turn=` row is the right grain, and the viewer's
   sum is the right display.
4. **Our example sessions were thin.** Before this work the fixtures showed
   user/assistant/tool/result/action/observation; they had no reasoning, no
   context, no timestamps, no ids. The imported sessions are the richer
   examples now; the every-turn-chip fixture still exists for the chips.

## Not done, on purpose

- No inbox integration: dropping a `.jsonl` in `inbox/` does not import it.
  `ingest` writes notes with `hick:transcript`; a conversation is a session.
  If people want the watch-folder, it is one line in the inbox matcher.
- No linking of `hick:tool-result` back to the file it read as `hick:read`
  (hash, commit, lines). Claude Code's Read results carry the text, not the
  hash; deriving `read` rows would mean hashing the repository at import
  time, which is a different claim than "this is what the model saw".
