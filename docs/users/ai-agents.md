# AI agents and hickory: built-in or bring your own

*For engineers deciding how AI should write and maintain their `.hick`
documents: the platform's built-in agent, or a coding agent they already use
(Claude Code, etc.) on a local repo.*

Either way, the end product is the same: agent work lands as `hick:session`
documents in `sessions/*.hick`, and `hickory promote` compacts a session into
a clean pipeline document. The choice is about *where the agent runs*, not
what it produces.

## Option 1: the built-in platform agent

```sh
export ANTHROPIC_API_KEY=...
hickory agent "add a section benchmarking sort vs awk" --doc docs/tour.hick
```

(In a cloud workspace, this is the web Agent panel — same loop, hosted.)

### Whose key it runs on

The agent runs on a key from one of four vendors — Anthropic, OpenAI,
DeepSeek, or xAI (Grok). Where that key comes from depends on where you are:

| Where | Which key | How to set it |
| --- | --- | --- |
| CLI | Yours, from the environment | `--provider grok` (or `HICKORY_LLM_PROVIDER`) selects the vendor; the matching `XAI_API_KEY` / `ANTHROPIC_API_KEY` / `OPENAI_API_KEY` / `DEEPSEEK_API_KEY` is read |
| Hosted, Open or Pro plan | Yours | Settings → API keys. Paste it once; it is checked against the vendor immediately and encrypted before it is stored |
| Hosted, Team or Business plan | Ours, from your plan's allowance | Nothing to do — unless you store your own key, in which case yours is used and the allowance is left alone |

Two properties worth knowing because they change what you have to do:

- **On a bring-your-own-key plan there is no fallback to our key.** With no key
  stored the agent answers "add one in Settings" rather than running on our
  account. That is the entitlement working, not an outage.
- **A stored key is write-only.** We show the last four characters and nothing
  else, and there is no endpoint that returns it — so keep your own copy from
  the vendor. Replacing a key is a paste, not a recovery.

With one key stored you are never asked which to use. Store a second and you
choose once; until you do, agent runs stop and say so.

What happens, procedurally:

1. The agent gets your prompt (and `--doc` context), then loops: it proposes
   a shell or python script, hickory executes it through the same `Executor`
   as `hickory run` (`HICKORY_EXECUTOR=local` or `=canopy`), and the
   observation goes back to the model.
2. Every turn — prompt, reasoning, each script, each observation — is
   appended to a `hick:session` document under `sessions/`. The session *is*
   the log; there is no separate chat transcript to lose.
3. When it's done, you review the session, then
   `hickory promote sessions/<name>.hick --out docs/benchmark.hick` to keep
   only the surviving pipeline (last write wins, dead ends dropped).

Choose this when you want sessions captured with full fidelity by
construction, or you're in the hosted app.

### The document edit tools (`--doc`)

Passing `--doc` makes that file the session's *primary document* and gives
the agent a first-class edit tool set alongside scripts. Tool calls are
`<hick:tool>` elements in the session file — the session stays a parseable,
replayable `hick:session` document.

The tools:

| Tool | What it does |
| --- | --- |
| `read_doc` | The document source, hashline-rendered: every line prefixed `hhhh\|`, a 4-hex content hash of that line. |
| `read_output` | A woven output file, hashline-rendered; `with_lineage` adds per-range annotations showing which lines are editable and where they come from. |
| `edit_output` | Edit a contiguous line run of an *output*; the edit maps back to the document byte-exactly through lineage. |
| `edit_doc` | The same edit shape applied directly to the document source. |
| `verify` | Execute the document for real (every exec block, every expectation) and write the output files. |

Edits anchor on content hashes (`run="firsthash..lasthash"`, or
`after="hash"` to insert), never on line numbers. That is what makes stale
edits impossible inside a session: after every successful edit the session
re-weaves immediately and hands back fresh hashes, and an anchor only
resolves against text that is actually there. If the document changes on
disk underneath the session (you, another tool), the session detects the
content mismatch, re-weaves once automatically, and either resolves the
edit against the fresh state or returns a structured error — it never
misapplies an edit.

The doctrine the agent follows (and that you can rely on when reviewing):

1. **Read both surfaces first** — `read_doc` and `read_output` with lineage.
2. **Code changes go through the output** (`edit_output`): the generated
   file is what the agent — like any engineer — actually reasons about, and
   lineage guarantees the document update reproduces the edit byte-for-byte.
3. **Structural and prose work goes through the document** (`edit_doc`):
   headings, copy blocks, pipeline structure.
4. **Lineage refusals are routing, not failure.** Editing exec output, a
   separator, or one occurrence of a copy block pasted twice cannot be
   reproduced exactly — the refusal names the document location to edit,
   and the agent follows the pointer with `edit_doc`.
5. **`verify` before done** — same semantics as `hickory run` + expectation
   checking, through the same executor as the agent's scripts.

## Option 2: bring your own coding agent (local repo)

Claude Code, Codex, Grok CLI — any agent that can run a command gets the
**same five tools** the built-in agent uses. It is not a lesser path.

`hickory init` sets this up:

1. A managed `<!-- HICKORY -->` section in `AGENTS.md` (which all three read;
   `CLAUDE.md` is pointed at it via `@AGENTS.md`) teaching the grammar, the
   golden rules, and the tool commands below.
2. A `hickory` entry in the project's `.mcp.json`, so an MCP-speaking harness
   picks the tools up on its own.
3. The pre-commit drift gate, as the backstop.

### The tools, as commands

```sh
hickory doc read <doc>                                  # source, each line prefixed hhhh|
hickory doc read-output <doc> --path f.rs --lineage     # a generated file + provenance
hickory doc edit-output <doc> --path f.rs --run aa12..bb34 < new.txt
hickory doc edit <doc> --run aa12 < new.txt
hickory doc verify <doc>
```

The `hhhh|` prefix on every line is a hash of that line's content, and it is
what edits anchor on — never line numbers. Two consequences you can rely on:
an edit built against a line that has since changed is **refused** rather
than landing somewhere wrong, and an edit made through a *generated* file is
mapped back into the document byte-exactly. A refusal from `edit-output`
names the document location to use with `edit` instead.

Add `--json` for `{"tool","ok","text"}` instead of bare text, which is easier
for an agent to branch on.

### The tools, over MCP

```sh
hickory mcp          # stdio MCP server; `hickory init` registers it in .mcp.json
```

Prefer this when your harness supports it. The server is one long-lived
process, so it keeps the edit session open between calls and re-weaves after
every edit — meaning a spent anchor is known to be stale immediately, without
re-reading anything. For a harness that keeps MCP config in its own global
file (Codex, Grok CLI), add a server named `hickory` running `hickory mcp`.

### Getting a session out of it

```sh
export HICKORY_SESSION=sessions/refactor.hick
```

Every tool call your agent makes — through either surface, across as many
processes as it likes — is appended to that `hick:session` document, and the
file is valid and parseable after each one. What is captured is what was done
to the document: the calls, the anchors, the text, the results. Your agent's
own reasoning stays in its own transcript; we don't read other tools' logs.

Choose this option when the `.hick` docs live inside a larger codebase and one
agent should handle both, or you want your existing agent tooling
(permissions, review flow, MCP servers).

## Mixing them

They compose. A common split: your coding agent does repo-wide work and
document editing; `hickory agent` does exploratory data/tool work where the
replayable session is the deliverable. Your coding agent can also *drive*
the built-in one (`hickory agent "..."`) and then review and promote the
resulting session — promote works on any `hick:session` file regardless of
which agent produced it.

## Don't assume

- **A coding agent's chat log is not a session document.** With
  `HICKORY_SESSION` set, an outside agent's *tool calls* are recorded as a
  real `hick:session` — but its reasoning is not, and work it does by editing
  files directly, without the tools, is invisible to the session entirely.
- **`promote` has nothing to do on a tool-driven session.** It reconstructs a
  pipeline from *script* writes; tool calls edit the document in place, so
  there is nothing left to promote. That is not a failure — the document is
  already the product, and the session is the record of how it got that way.
- **The built-in agent needs a key** — `ANTHROPIC_API_KEY` (or the selected
  provider's variable) on the CLI, a key stored in Settings in the hosted app.
  It executes scripts for real: under `HICKORY_EXECUTOR=local` that means
  unsandboxed, as your user.
- **"Encrypted at rest" is not "we never see it."** Your key is sealed in our
  database and unreadable from a dump, but the server holds it in memory to
  make the request — it has to. If that is not acceptable, use the CLI, where
  the key never leaves your machine.
- **`promote` is lossy on purpose**: it keeps the last write to each output
  and drops dead ends. Keep the original session file if you want the full
  history (it's just a file in git).
- **The edit session is single-writer, local.** It absorbs *file-level*
  external edits (content-hash detection + one automatic re-weave), which
  covers you saving the file in an editor between agent turns. What it does
  not do yet is merge with a *live* concurrent human editor keystroke-by-
  keystroke — that is the server-side follow-on (the hosted editor's CRDT
  layer), not a property of the local CLI session.
