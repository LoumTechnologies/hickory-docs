# A Claude Code Transcript Imports As A Session

Given a Claude Code transcript (`~/.claude/projects/<project>/<session>.jsonl`),
when `hick ingest --from claude-code <file>` runs, then a `hick:session` document is
written to `sessions/<date>-<title>.hick` that the product reads back as the
conversation it was: every prompt a turn with its `parent=` (the tree
survives), every reply's prose, reasoning, and tool calls with their full
input, every tool result, every token count — and what the harness put in
front of the model besides the conversation (compaction summaries, meta
prompts, hook output, attached files) as `hick:context`; the document parses,
opens in the app as bubbles with the work folded, and is offline, model-free,
and idempotent (the same transcript names one file). Everything not carried
is counted and printed, by reason; nothing is silently dropped.

The reason: a Claude Code conversation about a repository is the richest
record of why its files look the way they do, and it lives in a hidden
directory in a format nothing else reads. As a session document it sits in
the repository beside the work, opens like the dock's own conversations, and
counts as context provenance for `hick context` — the same standing as a
session the built-in agent wrote, because it is the same shape.

## The mapping

| Claude Code record | Session element |
|---|---|
| `user` record that is a prompt (not `isMeta`, not a compaction summary, no `tool_result`) | `<hick:user id= turn=<uuid> parent=<nearest prompt uuid above it> provider="anthropic" model= at=>` |
| `assistant` records sharing one `message.id` | one `<hick:assistant at= model= message=>` |
| `thinking` block | `<hick:reasoning>` (signature dropped, counted) |
| `text` block | prose |
| `tool_use` block | `<hick:tool name= call=>` with one `<hick:arg name=>` per input field |
| `user` record's `tool_result` block | `<hick:tool-result id= name= call= ok= at=>` |
| the message's `usage` | `<hick:usage turn= input= cache-write= cache-read= output=/>` after the assistant |
| `compact_boundary`, meta prompt, compaction summary, `system` with content, attachment, queued prompt | `<hick:context kind= at=>` |
| `ai-title` | root `title=`, and the file's slug |
| root | `<hick:session start= source="claude-code" session= cwd= branch= harness-version= title=>` |

**Dropped and counted**: image bytes (a placeholder names type and size),
thinking signatures, `total_tokens_reminder` / `task_reminder` attachments,
`turn_duration`, and bookkeeping records (`mode`, `permission-mode`,
`last-prompt`, `file-history-*`, `frame-link`, `atis-latch`,
`bridge-session`, `agent-name`, `relocated`, `worktree-state`).

## The no-escaping invariant

Only `hick:`-prefixed tags (and `<!--`) are structure. Four elements capture
their bodies verbatim — `hick:input`, `hick:tool-result`, `hick:reasoning`,
`hick:context` — and so carry anything. Prose and `hick:arg` do not, so:

1. A prompt or reply that quotes a hick tag (or `<!--`) is wrapped in
   `<hick:input>`; it still reads as the prompt or the prose.
2. A tool input with such a field is written as one `<hick:input>` holding the
   whole input as JSON.
3. A verbatim body that quotes its OWN close tag (a tool that printed a
   session file) has that exact token broken with a space (`</hick:input >`),
   the one byte-level change the import ever makes. Counted as "adapted".

## Boundary

Subagent transcripts (`<session>/subagents/agent-*.jsonl`) are separate files
and are not imported with their parent. Structured `toolUseResult` data is
not carried beyond the text the model saw. A converted document that does
not parse, or reads back as a different number of turns than prompts were
written, is NOT written to `sessions/`; `--stdout` still prints it so the
line can be found.

---

Last LLM verification:
- Date: 2026-08-23
- Reviewer: Claude Fable 5
- Result: verified. All 37 transcripts of this repository's own Claude Code
  history import (`hick ingest --from claude-code ~/.claude/projects/…/*.jsonl`),
  including the ones whose tool results and edits quote session files —
  the cases that exercise rules 1–3 above. The biggest (11,764 records)
  reads back as 74 turns, 2,125 replies, 2,059 tool calls and results, with
  3,591 dropped items reported by reason. Import is the same shape the dock
  writes, so it opens in the app as bubbles with the work folded.
- Evidence: `crates/hickory-cli/src/claude_code.rs` (`convert`, `prose`,
  `raw`, `tool_input`, `import_file`, `Converted::file_name`);
  `cmd_import` in `crates/hickory-cli/src/main.rs`; `hick:context` as a
  verbatim element in `crates/hick-lang/src/lib.rs::is_raw_content_tag`;
  `Step::Context` in `crates/hickory-agent/src/session_view.rs`; the editor's
  frame/fold/chip for `context` in `apps/web/src/editor/{wysiwyg,folding}.ts`
  and the dock's `StepView` in `apps/web/src/components/SessionTurns.tsx`.
- Tests: `claude_code::tests` (10) over
  `crates/hickory-cli/tests/fixtures/claude-code-small.jsonl` and inline
  transcripts — the read-back, the tree, ids and names, usage, the dropped
  counts, the three no-escaping rules, attribute quoting, file naming, and
  refusal of a foreign file. `hick_lang` tests cover verbatim capture.
- Caveat: the real-transcript run above is evidence of this date, not a
  test; the fixture is the test. Subagent files are not covered.
