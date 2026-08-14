# Literate debugging

A debugger answers "what was this value, here, at this moment". A document
answers "what is true about this code, checked". Literate debugging is the
place those meet: **the answer a debugger gives becomes part of the
document**, committed, re-derivable, and able to fail the build when it stops
being true.

This spec covers three surfaces:

1. **In a document** — declared captures that a run fills in, non-interactive.
2. **In the desktop app** — an interactive session a person drives, forwards
   and **backwards**.
3. **Over MCP** — the same session, driven by a coding agent.

One client and one channel serve all three. Behind them are two engines: a
recorded trace, which is where stepping backwards is possible at all, and a
live debug adapter for the languages a trace cannot reach.

And one rule holds across every surface: **the debugger never changes the
file.** Only a run does.

## Stepping backwards decides the architecture

A person wants to step back, and that single requirement settles most of the
design, because of a fact worth stating plainly:

> **You cannot step backwards in a live process.** The state is gone. Every
> reverse debugger that has ever existed either *recorded* execution or
> *re-runs it from a checkpoint*.

So "interactive time-travel debugging" is not a debugger feature bolted on. It
is a **recording**, played. And a recording is an artifact — which is exactly
what this product is already about. The awkward requirement turns out to be
the one that fits best.

That gives two tiers, and they are honestly different.

### Tier 1 — recorded execution (the default, and where time travel lives)

Every language worth debugging here already exposes an in-process tracing
hook: Python's `sys.settrace` / `sys.monitoring`, Node's inspector, Ruby's
`TracePoint`, PHP's tick handlers. A small per-language shim installs one and
emits a stream of events — frame entered, line reached, values changed, frame
left — as the cell runs.

Three things fall out, all of them good:

- **Step back is an index.** Forwards and backwards are the same operation on
  a recorded sequence: move the cursor. No re-execution, no checkpoints, no
  guessing. The same is true of "run backwards to where this variable last
  changed", which a live debugger cannot offer at all.
- **The sandbox stops being a problem.** Tracing is in-process, so there is no
  `ptrace`, no attaching, no PID-namespace obstacle, and no privilege to relax.
  The spec's ugliest risk simply disappears for this tier.
- **The recording bounds itself.** We know exactly which files the document
  generated, so tracing is scoped to *those* frames by default. A step never
  descends into the standard library, the trace stays small, and what you are
  stepping through is only ever the document's own code — which is the
  literate reading of a stack anyway.

Its limits are equally real: it does not cover compiled languages, and
recording every line of a hot loop is expensive. Both are addressed below.

### Tier 2 — a live DAP session (compiled languages, arbitrary depth)

The Debug Adapter Protocol is LSP's sibling: JSON-RPC over stdio, one adapter
per ecosystem, already implemented for everything that matters — `debugpy`,
`js-debug`, Delve, CodeLLDB, `java-debug`, `rdbg`. We already ran this exact
play for LSP and it worked, so adapters get the same treatment: a per-language
candidate table, discovery that prefers what the project pins, an installer
that is never automatic, and the position mapping between document and virtual
file that is **already written and tested**.

DAP has reverse requests in the protocol — `stepBack`, `reverseContinue`,
behind a `supportsStepBack` capability — but almost no adapter implements
them, because of the fact above. Where it works it is because something is
recording underneath: Delve over `rr` for Go, `rr` itself for Rust and C++ on
Linux/x86, WinDbg's time-travel traces on Windows. So for compiled languages
the honest position is:

- Forward stepping: everywhere an adapter exists.
- Backward stepping: Linux, x86, `rr` present — **or not at all**, reported as
  a missing capability rather than a button that does nothing.

A session advertises what it can do and the app greys out the rest. A debugger
that offers a control which silently fails is worse than one that says no.

### One recording, two features

The tier-1 recorder is also what fills in a document's declared captures: a
capture is **a query over a recording**, not a separate mechanism. Record the
cell, then ask "what was `subtotal` at `pricing.py:14`, each time through".
Interactive stepping and non-interactive capture are the same machinery read
two ways, which is why this is one spec and not two.

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
in the gutter of the document, step over / into / out, **step back** and
**run backwards to the last change of a variable**, a variables pane, a call
stack, and an expression evaluator. Because tier 1 is a recording, the reverse
controls are not a special mode — a session opens on a recorded run and the
cursor moves either way.

### The debugger cannot touch the file

This is the guarantee that makes an interactive debugger safe to have in a
tool whose whole point is that the file is the truth:

> **A session is a reader.** No sequence of stepping, evaluating, or poking at
> variables can change the document, the files it generates, or the recorded
> transcripts. The only things that write are `hick run` and a person editing.

It is architectural rather than a matter of care:

- **A session has no write path.** The debug channel carries no edit
  operations. Document edits go through the CRDT room and the hashline edit
  API, which the debug bridge does not hold.
- **A session runs in a scratch clone of the container**, and its outputs are
  discarded at the end. Stepping through a cell that writes `report.csv`
  produces a `report.csv` in the session's own workdir and nowhere else —
  `hick run` copies outputs back, a session never does.
- **Transcripts are untouched.** A session records nothing into the document's
  cached transcripts, so a cell's committed baseline cannot drift because
  somebody debugged it.
- **Evaluation can still mutate the program's own state** — `debug_eval` of
  `lines.pop()` really pops — and that is the debugged process's business, not
  the document's. It dies with the session.

Promotion is the one path from a session into the document, and it is an
**edit the person makes**, not something the debugger does: stopped at a
breakpoint with `subtotal = 19.99` visible, one action drafts a
`<hick:capture>` with an `<hick:expect>` around it and applies it through the
ordinary edit path, where it is undoable and reviewable like anything they
typed. The thing you just learned becomes the thing the document checks.

**Transport.** The socket already multiplexes by channel byte: `0x00` Yjs,
`0x01` runs, `0x02` LSP. Debugging is `0x03`, carrying DAP messages the same
way `0x02` carries LSP ones, with a bridge that owns the session's lifetime
and rewrites paths between the client's scheme and the filesystem — the same
job `LspBridge` does, and worth writing as a sibling rather than a
generalisation, because the two protocols' initialisation handshakes differ
more than they look.

A tier-1 session speaks the same DAP shapes over that channel even though
there is no adapter process behind it, so the app has one client. `stepBack`
and `reverseContinue` are answered from the recording; a tier-2 session
forwards them, or reports `supportsStepBack: false` and the app greys the
controls out.

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
| `debug_state` | Where it is stopped, the stack, the frame's variables, and which controls this session supports. |
| `debug_eval` | Evaluate an expression in a chosen frame. |
| `debug_step` | `over` \| `into` \| `out` \| `continue` \| `back` \| `reverse` \| `back_to_change` (of a named variable). |
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

Only tier 2 pays this, which is a large part of why tier 1 is the default.

**Tier 1 costs nothing.** The tracer is in-process — the cell imports a shim
and installs its own language's hook — so there is no `ptrace`, no attaching,
no PID namespace to cross, and no privilege to relax. A recorded session is
exactly as confined as the run it recorded, because it *is* that run.

**Tier 2 has a real problem.** A debug adapter controls another process:
`ptrace` on Linux, `task_for_pid` on macOS. The sandbox gives each cell its
own PID namespace (`--unshare-all`), so an adapter outside cannot see the
process at all. The resolution is that the adapter goes **inside**, launched
as the cell's entry point rather than attached from outside:

```
bwrap … -- debugpy --listen … -- python3 pricing.py
```

It then shares the namespace with its target and needs no extra privilege.
What that costs:

- The adapter must be **visible inside the sandbox**, so it is bound read-only
  into the cell's empty `$HOME`, joining the toolchain directories already
  bound there for the same reason `duckdb` needed to be.
- **`--unshare-pid` may still block ptrace between siblings** on hardened
  kernels. If it does, the choice is a per-cell relaxation (stated in
  `describe()`, never silent) or refusing to debug under confinement.
  Refusing is the default; a sandbox that quietly weakens itself for a feature
  is worse than a feature that says it needs `HICKORY_EXECUTOR=local`.
- **`rr`**, which is what makes tier-2 reverse stepping possible at all, needs
  performance counters that are commonly unavailable in containers and on
  cloud VMs. Detect and report, exactly as `Sandbox::detect` learned to.
- **Windows AppContainer** and debugging is a genuine unknown, on a path that
  has never been run at all. Assume it does not work until someone tries it.

## What recording costs

The honest limits of tier 1, since they decide when tier 2 is worth its price:

- **Speed.** A line-granularity trace of a hot loop is orders of magnitude
  slower. Mitigated by scoping to the document's own generated files (never
  library internals), by a `granularity` of `line` or `function`, and by
  recording *changes* rather than whole frames.
- **Size.** A trace is bounded by a byte budget with a ring buffer, so a long
  run keeps its most recent window rather than exhausting memory. A session
  says plainly when it is looking at a truncated recording; silently showing
  half a run would be the worst outcome.
- **Values that cannot be recorded.** Open sockets, file handles, and anything
  whose `repr` runs code are recorded as an opaque marker, not evaluated.
- **Compiled languages are not covered at all.** Rust, Go and C get tier 2 or
  nothing, and reverse stepping there needs `rr`.

## Order of work

Ordered so the cheap parts prove the expensive ones are wanted.

1. **`<hick:capture>` by wrapping, no tracer and no DAP.** Print the named
   expressions at the named point. Any language that can print, no adapter,
   no ptrace, no sandbox change — and it delivers the actual prize
   (assertions about intermediate state) for about a day's work.
2. **The tier-1 recorder**, Python first: a shim, a trace format, and the
   capture step re-implemented as a query over a recording. At this point a
   recording exists but nothing plays it.
3. **Channel `0x03` and the app's session UI**, playing a tier-1 recording:
   breakpoints, forwards, **backwards**, variables, evaluation against a
   recorded frame, and promotion into the document. This is the step where a
   person can actually use it, and where the isolation guarantee gets its
   tests.
4. **The recorder for the other traced languages** — Node, Ruby — which is
   one shim each once the trace format is settled.
5. **Tier 2**: adapter discovery, `hick dap install`, and live sessions for
   compiled languages, forwarding the same channel.
6. **MCP tools**, once a human has driven the session API first.

Steps 2, 3 and 5 are each worth roughly what this week's LSP work was.

## What this is not

- **Not a profiler.** Sampling and timing are a different tool with different
  determinism problems.
- **Not remote debugging.** Same machine, same rules as everything else here:
  no server we operate.
- **Not a replacement for `<hick:expect>`.** Most claims are about a cell's
  output and should stay that way. Captures are for claims that can only be
  made from inside a function.
