# Literate debugging

A debugger answers "what was this value, here, at this moment". A document
answers "what is true about this code, checked". Literate debugging is the
place those meet: **the answer a debugger gives becomes part of the
document**, committed, re-derivable, and able to fail the build when it stops
being true.

This spec covers three surfaces over one mechanism:

1. **In a document** — declared captures that a run fills in, non-interactive.
2. **In the desktop app** — an interactive session a person drives.
3. **Over MCP** — the same session, driven by a coding agent.

They are one implementation. The differences are who is stepping and whether
anything is written back.

## Why DAP

The Debug Adapter Protocol is LSP's sibling: a JSON-RPC surface over stdio,
one adapter per language ecosystem, already implemented for everything that
matters — `debugpy` (Python), `js-debug` (Node), Delve (Go), CodeLLDB
(Rust, C, C++), `java-debug`, `rdbg` (Ruby).

We already run the same play for LSP, and it worked: a per-language candidate
table, discovery that prefers what the project pins, an installer that is
never automatic, and a meta-server that maps positions between the document
and the virtual files. Debugging needs exactly those four things again, and
almost all of the hard part — the position mapping — is written and tested.

**This is not a second execution path.** A debugged cell is the same cell, in
the same container, under the same executor and the same sandbox. It is
launched with an adapter in front of it.

## 1. In a document: declared captures

A cell can declare what to look at. Nothing is stepped and nobody waits:

```xml
<hick:exec container="lab">
python3 pricing.py
  <hick:capture at="pricing.py:14" of="subtotal, len(lines)" />
</hick:exec>
```

`at` is a location in a **generated file**, which is a location in the
document — the same mapping `hick-lsp` already does to turn a document
position into a virtual-file position, run in the same direction. `of` is a
list of expressions evaluated in that frame.

On a run, the adapter sets a breakpoint there, and on every hit evaluates the
expressions, records them, and **continues**. The run never blocks. What comes
back is a table, woven like any other transcript:

| hit | `subtotal` | `len(lines)` |
|-----|-----------|--------------|
| 1   | `0`       | `3`          |
| 2   | `19.99`   | `3`          |

And because it is recorded output, `<hick:expect>` pins it. A document can now
say *"`subtotal` is 0 on the first pass and never negative"* and fail the day
that stops being true — a claim about a value **inside** a function, which no
amount of stdout checking reaches.

That is the actual prize. Not stepping: **assertions about intermediate state,
in version control, re-derived on every run.**

### Determinism is the whole risk

A captured value that changes every run is a drift generator, and a document
full of them trains people to ignore drift. Captures are therefore:

- **Expressions, not frame dumps.** You name what you want. A whole locals
  dump would include addresses, iterators and timestamps.
- **Redacted by the same rules as volatile outputs**, which this repo already
  has for exactly this reason.
- **Ordered by hit index, not by wall-clock.**

A capture that cannot be made deterministic belongs in an interactive session
instead, which writes nothing.

## 2. In the desktop app: an interactive session

The app is where a person sits, so the app gets a real debugger: breakpoints
in the gutter of the document, step over/into/out, a variables pane, a call
stack, and an expression evaluator.

The distinction that keeps this coherent with everything else:

> A **document run** is never interactive — that is what makes an unattended
> or scheduled run safe. A **session** is a person driving one cell, and it
> writes nothing to the document unless they promote a capture into it.

Promotion is the interesting verb. You are stopped at a breakpoint, you see
`subtotal = 19.99`, and one action turns that into a `<hick:capture>` with an
`<hick:expect>` around it — the thing you just learned becomes the thing the
document checks from now on. That is the literate part; the stepping is
ordinary.

**Transport.** The socket already multiplexes by channel byte: `0x00` Yjs,
`0x01` runs, `0x02` LSP. Debugging is `0x03`, carrying DAP messages the same
way `0x02` carries LSP ones, with a bridge that owns the adapter's lifetime
and rewrites paths between the client's scheme and the filesystem — the same
job `LspBridge` does, and worth writing as a sibling rather than a
generalisation, because the two protocols' initialisation handshakes differ
more than they look.

**Positions.** Breakpoints are set in document coordinates and travel to the
adapter in virtual-file coordinates; stack frames come back the other way. A
frame in a file the document did not generate — a library, the standard
library — is shown as read-only and never mapped, exactly as LSP "go to
definition" already handles a target outside the document.

## 3. Over MCP: the same session, for an agent

An agent that can only read a failing document is guessing. An agent that can
stop at the failure and ask what a variable held is doing what a person does.

The tool surface is deliberately small, and mirrors the existing document
tools in style — small verbs, explicit ids, no hidden session state beyond a
handle:

| tool | what it does |
|---|---|
| `debug_start` | Launch a cell under its adapter. Returns a session id. Breakpoints given up front, in document coordinates. |
| `debug_state` | Where it is stopped, the stack, and the frame's variables. |
| `debug_eval` | Evaluate an expression in a chosen frame. |
| `debug_step` | `over` \| `into` \| `out` \| `continue`. |
| `debug_stop` | End the session. |

Two rules that matter more than the surface:

- **A session is bounded.** It ends on its own after an idle timeout and when
  the agent's turn ends. A debugger that outlives the conversation is a
  process holding a workdir open, and the first symptom is a later run failing
  for no visible reason.
- **The agent's session is as confined as a cell.** Same sandbox, same
  capabilities, same denied network. An "attach to my process" tool that
  escaped the sandbox would undo the reason the sandbox exists.

## Adapters are installed exactly like language servers

`hick dap install python`, and the same three rules that made
`hick lsp install` defensible:

- **Discovery first.** `debugpy` in the project's virtualenv, `js-debug` in
  `node_modules`, Delve on `PATH`, CodeLLDB in a VS Code extensions
  directory — whatever is already there wins, and nothing is installed to use
  what exists.
- **Never automatic.** An install reaches the network and runs the package's
  setup scripts. It happens when a person types it.
- **Confined.** Into `.hick-cache/adapters/`, through the same sandbox, with
  the same installer profile that keeps the toolchain visible and redirects
  `HOME` into the prefix.

And the same failure mode is worth pre-empting: `hick lsp install` shipped
with a broken package pair for a week because `typescript` on npm became the
native port with no `tsserver.js`. An adapter that is installed but cannot
start must be **passed over at discovery**, not spawned to fail later.

`<hick:needs>` extends here naturally: a document that declares captures needs
its adapter, and saying so before the run is better than a DAP error nobody
can read.

## What the sandbox costs us

This is the part with real risk, and it is worth stating before anyone starts.

A debug adapter controls another process: `ptrace` on Linux, `task_for_pid` on
macOS. The sandbox currently gives each cell **its own PID namespace**
(`--unshare-all`), which means a debugger outside the sandbox cannot see the
process at all.

The resolution is that **the adapter goes inside**, launched as the cell's own
entry point rather than attached from outside:

```
bwrap … -- debugpy --listen … -- python3 pricing.py
```

The adapter then shares the namespace with its target and needs no additional
privilege. DAP travels over the confined process's stdio, which the executor
already pipes. What this costs:

- The adapter must be **visible inside the sandbox** — so it is bound
  read-only into the cell's empty `$HOME`, joining the toolchain directories
  already bound there for the same reason `duckdb` needed to be.
- **`--unshare-pid` may still block ptrace between siblings** on hardened
  kernels. If it does, the choice is a per-cell relaxation (stated in
  `describe()`, never silent) or refusing to debug under confinement. Refusing
  is the default; a sandbox that quietly weakens itself for a feature is worse
  than a feature that says it needs `HICKORY_EXECUTOR=local`.
- **Windows AppContainer** and debugging are a genuine unknown here, and the
  AppContainer path has never been run at all. Assume it does not work until
  someone tries it.

## Order of work

1. **`hick:capture` with no DAP at all.** Wrap the cell to print the named
   expressions at the named point. Works in any language that can print, needs
   no adapter, no ptrace, no sandbox change — and delivers the actual prize
   (assertions about intermediate state) for the price of a shell wrapper.
   Ship this first and find out whether the rest is wanted.
2. **`hick-dap`**: adapter discovery, install, and a session that can set a
   breakpoint, evaluate, and continue. Non-interactive capture moves onto it.
3. **Channel `0x03` and the app's session UI.**
4. **MCP tools**, once the session API has been driven by a human first.

Steps 2–4 are each worth roughly what the LSP work was, and step 1 is worth
about a day. The ordering is deliberate: step 1 is the only one that can be
proven useful before the expensive parts are built.

## What this is not

- **Not a profiler.** Sampling and timing are a different tool with different
  determinism problems.
- **Not remote debugging.** Same machine, same rules as everything else here:
  no server we operate.
- **Not a replacement for `<hick:expect>`.** Most claims are about a cell's
  output and should stay that way. Captures are for claims that can only be
  made from inside a function.
