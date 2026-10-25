<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/users/ai-agents.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `fe986576f56e4740ba577b287faabe1bc33d9e0ed0c70ea743bb7f103b75be23`.

<book:entry path="docs/users/ai-agents.md">
# AI agents and hick: built-in or bring your own

*For engineers deciding how AI should write and maintain their `.md`
documents: the built-in `hick agent`, or a coding agent they already use
(Claude Code, etc.) on a local repo.*

Agent edits made through Hickory's tools change the source document in place.
Session documents record how the work happened. The built-in Python agent proposes reviewed document changes and submits a
literate artifact that Hickory checks against the captured workspace.

## Use your existing agent inside Hickory

Choose **Codex** or **Claude Agent** in the Agent pane. Hickory connects through
ACP, supplies its document-editing tools, and records the conversation as a
session document. See [Use Codex in Hickory Docs](acp-agents.md) for installation,
sign-in, permissions, and recovery.

## Option 1: the built-in Python agent

The macOS desktop app bundles its runtime. Add your provider key in Settings,
select **Hickory Python**, and send a task. A source build needs an installed
Python runtime configured in Settings → Agents. Each edit opens a review:
choose Accept change or Reject change. Stop cancels pending review.

From a terminal:

```sh
hick agent --python /absolute/path/to/python \
  --doc docs/tour.md "add a section benchmarking sort vs awk"
```

See [Python runtime setup](../developers/python-agent.md) for installation and
running the Python package independently. The CLI reads keys configured in
Hickory Settings or provider environment variables. `--provider` and `--model`
select the provider and model. Missing credentials or confinement stop execution.

Python owns the agent loop. Model snippets run in Monty, inside a separately
confined process, and call Hickory tools. They cannot execute a shell, write
workspace files directly or invoke unrestricted verification commands. `HICKORY_EXECUTOR=local`
does not disable this confinement. Hosted Windows support remains unavailable.

In an attached application window, Python and ACP agents can also discover
registered controls with `ui_observe` and invoke their current `ui_act` handles.
Current coverage includes cell Run/source/transcript controls, menu disclosure,
environment setup, debugger state/stepping/breakpoints, runtime/profile reads,
terminal supervision and bounded output/input, editor reads/selections/folds,
source-guarded saved-document execution, review reading and coach controls.
Setup and execution return actual job IDs; inspect their receipts before judging
a result. Editor reads are bounded observations, not edit baselines. Terminal
prompts and human review resolution require actual human input. Agent comments
and investigations retain agent-declared origin. Other
UI capabilities are still being connected. Required Save input and unavailable
controls are reported explicitly. CLI runs without an attached window have no
UI controls. Human-only edit approvals remain separate.

The agent reads documents and woven outputs, then proposes hash-anchored
`edit_doc`, `edit_output` or `create_doc` changes. Hickory requires approval of
the exact preview and rejects detected source changes during review. Unsaved
editor text is context; save untitled notes before requesting tool edits.

Prompts, reasoning, model requests, actual tools and approval decisions remain
in a session document. Python decides when to submit its final literate changeset;
Hickory checks that it reproduces captured output bytes. Successful completion
means source-output equality, with runtime checks unrun. Run project checks
explicitly when needed. A model claim is not verification evidence.

## Option 2: bring your own coding agent (local repo)

Claude Code, Codex, Grok CLI — any agent that can run a command gets the
shared document tools, including explicit verification. External agents retain
their own command permissions. The built-in Python agent uses confined snippets
and registered host controls; direct shell access remains unavailable and exact
edit review remains required.

`hick init` sets this up:

1. A managed `<!-- HICKORY -->` section in `AGENTS.md` (which all three read;
   `CLAUDE.md` is pointed at it via `@AGENTS.md`) teaching the grammar, the
   golden rules, and the tool commands below.
2. A `hick` entry in the project's `.mcp.json`, so an MCP-speaking harness
   picks the tools up on its own.
3. The pre-commit drift gate, as the backstop.

### The tools, as commands

```sh
hick doc read <doc>                                  # source, each line prefixed hash|
hick doc read-output <doc> --path f.rs --lineage     # a generated file + provenance
hick doc edit-output <doc> --path f.rs --run aa12..bb34 < new.txt
hick doc edit <doc> --run aa12 < new.txt
hick doc verify <doc>
```

The `hash|` prefix on every line is a hash of that line's content, and it is
what edits anchor on — never line numbers. Two consequences you can rely on:
an edit built against a line that has since changed is **refused** rather
than landing somewhere wrong, and an edit made through a *generated* file is
mapped back into the document byte-exactly. A refusal from `edit-output`
names the document location to use with `edit` instead.

Add `--json` for `{"tool","ok","text"}` instead of bare text, which is easier
for an agent to branch on.

### The tools, over MCP

```sh
hick mcp          # stdio MCP server; `hick init` registers it in .mcp.json
```

Prefer this when your harness supports it. The server is one long-lived
process, so it keeps the edit session open between calls and re-weaves after
every edit — meaning a spent anchor is known to be stale immediately, without
re-reading anything. For a harness that keeps MCP config in its own global
file (Codex, Grok CLI), add a server named `hick` running `hick mcp`.

### Getting a session out of it

```sh
export HICKORY_SESSION=sessions/refactor.md
```

Every tool call your agent makes — through either surface, across as many
processes as it likes — is appended to that `hick:session` document, and the
file is valid and parseable after each one. What is captured is what was done
to the document: the calls, the anchors, the text, the results. Your agent's
own reasoning stays in its own transcript.

### Importing the transcript itself (Claude Code)

Claude Code keeps the whole conversation — prompts, replies, reasoning, every
tool call and result, token counts — in
`~/.claude/projects/<project>/<session-id>.jsonl`. Bring it in as a session
document:

```sh
hick ingest --from claude-code ~/.claude/projects/-home-me-notes/0509e08e-….jsonl
# → sessions/20260813-133534-revise-home-page-to-prioritize-agent-fir.md
```

Offline, no model, nothing deleted. The result opens in the app as the
conversation (bubbles, the agent's work folded), keeps the turn tree, and
counts as context provenance for `hick context`. The same transcript imported
twice names one file and is skipped (`--force` overwrites). What is left out
— image bytes, thinking signatures, the harness's token reminders and
bookkeeping — is printed, by reason, every time. `--stdout` prints the
document instead of writing it. Details and the full mapping:
`docs/specs/freeform/claude-code-sessions.md`.

Choose this option when the `.md` docs live inside a larger codebase and one
agent should handle both, or you want your existing agent tooling
(permissions, review flow, MCP servers).

## Mixing them

External agents can do repo-wide command work using their own permissions.
The built-in Python agent uses reviewed document tools and retains its session
and verified changeset artifact. Historical script sessions can still be
imported or promoted; new Monty snippets are not shell pipeline recordings.

## Don't assume

- **An agent used outside Hickory's ACP pane has no session conversation until you import
  it.** With `HICKORY_SESSION` set, an outside agent's *tool calls* are
  recorded as a real `hick:session` — but its reasoning is not, and work it
  does by editing files directly, without the tools, is invisible to that
  session. `hick ingest --from claude-code` is the other half: the transcript itself,
  reasoning and all, after the fact.
- **`promote` has nothing to do on a tool-driven session.** It reconstructs a
  pipeline from *script* writes; tool calls edit the document in place, so
  there is nothing left to promote. That is not a failure — the document is
  already the product, and the session is the record of how it got that way.
- **The built-in agent needs a provider key and a configured or bundled runtime.**
  It stays confined regardless of the cell-executor setting. Its completion
  artifact does not establish native runtime correctness.
- **`promote` is lossy on purpose**: it keeps the last write to each output
  and drops dead ends. Keep the original session file if you want the full
  history (it's just a file in git).
- **The edit session is single-writer, local.** It absorbs *file-level*
  external edits (content-hash detection + one automatic re-weave), which
  covers you saving the file in an editor between agent turns. What it does
  not do yet is merge with a *live* concurrent human editor keystroke-by-
  keystroke — that is the desktop app's editor (its CRDT buffer), not a
  property of the CLI session.

For a durable account of assumptions, decisions, historical inputs and executed
checks, see [Revisit why an agent did something](debugging-agent-decisions.md).
Pass the read's whole-surface `snapshot` to document edit tools (`--snapshot` on
`hick doc edit`/`edit-output`) to protect changes inside a range as well as its
endpoints. Legacy short anchors no longer resolve; read fresh SHA-256 anchors.
</book:entry>

</book:doc>
