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

On a local git repo, `hickory init` makes a general coding agent — Claude
Code is the tested one — a first-class alternative:

1. `hickory init` writes a managed `<!-- HICKORY -->` section into
   `AGENTS.md` (and points `CLAUDE.md` at it via `@AGENTS.md`). The section
   teaches the agent the hick grammar essentials and the golden rules:
   - edit `.hick` sources, never generated outputs (unless using lineage
     tooling);
   - after editing, run `hickory run <doc>` then `hickory check <doc>`;
   - sessions live in `sessions/*.hick`.
2. The agent edits documents like any other source file, with `hick-lsp`
   available for diagnostics and the CLI for ground truth.
3. The pre-commit hook from `hickory init` is the backstop: if the agent
   commits a drifted document, the commit fails with a diff — the same gate
   you have.

Choose this when the `.hick` docs live inside a larger codebase and one
agent should handle both, or you want your existing agent tooling
(permissions, MCP servers, review flow).

## Mixing them

They compose. A common split: your coding agent does repo-wide work and
document editing; `hickory agent` does exploratory data/tool work where the
replayable session is the deliverable. Your coding agent can also *drive*
the built-in one (`hickory agent "..."`) and then review and promote the
resulting session — promote works on any `hick:session` file regardless of
which agent produced it.

## Don't assume

- **A coding agent's chat log is not a session document.** Only
  `hickory agent` (or the hosted Agent panel) produces replayable
  `hick:session` files. Claude Code's work shows up as ordinary commits.
- **The built-in agent needs `ANTHROPIC_API_KEY`** and executes scripts for
  real — under `HICKORY_EXECUTOR=local` that means unsandboxed, as your user.
- **`promote` is lossy on purpose**: it keeps the last write to each output
  and drops dead ends. Keep the original session file if you want the full
  history (it's just a file in git).
- **The edit session is single-writer, local.** It absorbs *file-level*
  external edits (content-hash detection + one automatic re-weave), which
  covers you saving the file in an editor between agent turns. What it does
  not do yet is merge with a *live* concurrent human editor keystroke-by-
  keystroke — that is the server-side follow-on (the hosted editor's CRDT
  layer), not a property of the local CLI session.
