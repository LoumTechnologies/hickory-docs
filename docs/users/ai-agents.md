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
