# Literate debugging

A debugger answers "what was this value, here, at this moment". A document
answers "what is true about this code, checked". Literate debugging is the
place those meet: **the answer a debugger gives can become part of the
document**, committed, re-derived on every run, and able to fail the build
when it stops being true.

Three surfaces, one engine:

1. **In the desktop app** — an interactive session: breakpoints, continue,
   step over / in / out, and arbitrary expressions.
2. **In a document** — the same breakpoints and the same expressions, with
   nobody stepping: the run stops, evaluates, records, and continues.
3. **Over MCP** — the same session, driven by a coding agent.

And one rule across all three: **the debugger never changes the file.** Only a
run does.

## One engine: DAP

The Debug Adapter Protocol is LSP's sibling — JSON-RPC over stdio, one adapter
per ecosystem, already implemented for everything that matters: `debugpy`
(Python), `js-debug` (Node, TypeScript), Delve (Go), CodeLLDB (Rust, C, C++),
`java-debug`, `rdbg` (Ruby).

Everything asked for here is in the *universally implemented* core of that
protocol, which is what makes the plan cheap:

| what you want | DAP request | implemented by |
|---|---|---|
| breakpoints | `setBreakpoints` | every adapter |
| resume | `continue` | every adapter |
| step over | `next` | every adapter |
| step in | `stepIn` | every adapter |
| step out | `stepOut` | every adapter |
| evaluate an expression | `evaluate` | every adapter |
| call stack, locals | `stackTrace`, `scopes`, `variables` | every adapter |

None of that is optional or adapter-specific. It is the part of DAP every
editor's debugger UI is built on.

**Reverse stepping is out of scope**, and it was the only thing that would
have cost real money: you cannot step backwards in a live process, so it needs
execution recorded or replayed from checkpoints — a tracer per language, a
trace format, ring buffers, a second engine. Dropping it removes all of that.
The note for a future reader: it would need `rr` (Linux, x86, performance
counters) for compiled languages and an in-process tracer for the rest, and
neither shares much with what is described here.

We already ran this play for LSP and it worked, so adapters get the same
treatment: a per-language candidate table, discovery that prefers what the
project pins, an installer that is never automatic, and the mapping between
document and virtual-file positions that is **already written and tested**.

A debugged cell is the same cell, in the same container, under the same
executor and the same sandbox. It is launched with an adapter in front of it —
not a second execution path.

## 1. In the app: an interactive session

Breakpoints in the document's gutter, set on the lines of a `hick:file` block
and travelling to the adapter in virtual-file coordinates. Continue, step over,
step in, step out. A call stack, a variables pane, and an expression box that
evaluates **in the frame you have selected** — `evaluate` with that frame's
id, so `self.lines[2].price` means what it means *there*, not what it would
mean at the top of the file.

### What the surface should be

JetBrains is the standard to measure against, so this is what to take from it,
in the order it pays off. Everything here is DAP; nothing needs a protocol
extension.

**The gutter.**

- A **red dot** on any line with a breakpoint. Click the gutter to toggle.
- A **hollow dot** when the adapter could not verify it — `setBreakpoints`
  answers with `verified`, and a breakpoint on a line the debugger will never
  reach should look different from one that works. Most editors show this and
  it saves an afternoon.
- A **dot with a `?`** for a conditional breakpoint, its condition on hover.
- **Muted breakpoints** (a global toggle) rather than deleting them to get one
  clean run.
- The **paused line** marked distinctly from the breakpoints — a filled arrow
  in the gutter and a tinted line — because "where I stopped" and "where I
  asked to stop" are different facts and the second is often several lines
  away after a step.

**Inline values, which is the single biggest thing JetBrains does.** While
paused, each line shows its variables' current values greyed at the end of the
line: `subtotal = 19.98`. It removes most of the reason to look at a variables
pane at all, and it is cheap here — `scopes` + `variables` for the paused
frame, matched to identifiers on the visible lines. This is also where the
notebook can beat a conventional IDE, because the document's prose sits right
beside the code the values belong to.

**Hover, which is two facts at once.** Not paused, a hover is what LSP already
gives: the symbol's type and doc. Paused, it must ALSO show the runtime value,
and both together rather than one replacing the other — the type is what it
should be, the value is what it is, and a debugger exists for the moments
those disagree. A composite object expands in place, lazily, through
`variables` on the reference the adapter returned.

The plumbing already points this way: the hover tooltip merges sources today —
a diagnostic first, then type information under a rule. Runtime values are a
third band in the same tooltip, above both.

**The panes.** Frames (with the document's own frames visually separated from
library ones), variables as a lazy tree, watches that persist across sessions,
and threads only when there is more than one.

**Beyond the five verbs**, in the order they earn their keep:

- **Run to cursor**, which is the step you actually want most of the time.
- **Evaluate expression** as a dialog, not just an inline box, so a long
  expression is editable.
- **Set value** — edit a variable in the pane and continue. Honest note: this
  mutates the debugged program, which is fine in a session and is exactly why
  a session may never write to the document.
- **Smart step-into**: when a line has several calls, ask which one. DAP has
  this as `stepInTargets`, and it is a small feature that removes a whole
  class of "I stepped into the wrong thing, start again".
- **Drop frame** — DAP's `restartFrame`. Pop the current frame and re-enter
  the function from its first line.

  **This is worth flagging against what was said earlier**: stepping backwards
  needs recorded execution, and drop-frame is not that — it re-runs rather
  than rewinds, so side effects already performed stay performed. But it
  answers most of what people actually want from "go back": *I stepped one
  too far, let me do that function again*. It costs nothing extra and it is
  the honest 80% of reverse debugging.
- **Exception breakpoints** — break where a throw originates rather than
  where it surfaced.
- **Field watchpoints** — break when a value changes. `debugpy` and Java
  support this; most do not, so it is capability-gated like everything else.

**Capability-gated, always.** DAP advertises what an adapter supports, and a
control that is present but silently does nothing is worse than one that is
absent. The app reads the capability and greys out the rest, per adapter.

Two details that matter more than the controls:

**Stepping into a frame the document did not generate** — a library, the
standard library — shows that frame read-only rather than pretending it is
part of the document. The same thing LSP "go to definition" already does with
a target outside the document.

**A conditional breakpoint is the one worth having.** DAP's `condition` and
`hitCondition` are widely implemented, and "stop when `subtotal < 0`" is the
question people actually have. It is also what makes the non-interactive form
below useful rather than noisy.

### The debugger cannot touch the file

This is the guarantee that makes an interactive debugger safe in a tool whose
whole point is that the file is the truth:

> **A session is a reader.** No sequence of stepping, evaluating, or poking at
> variables can change the document, the files it generates, or the recorded
> transcripts. The only things that write are `hick run` and a person editing.

Architectural rather than a matter of care:

- **The debug channel carries no edit operations.** Document edits go through
  the CRDT room and the hashline edit API, which the debug bridge does not
  hold.
- **A session runs in a scratch clone of the container**, and its outputs are
  discarded when it ends. Stepping through a cell that writes `report.csv`
  produces one in the session's own workdir and nowhere else — `hick run`
  copies outputs back, a session never does.
- **Transcripts are untouched**, so a cell's committed baseline cannot drift
  because somebody debugged it.
- **Evaluation can still mutate the debugged program** — `lines.pop()` really
  pops — and that is that process's business. It dies with the session. It is
  also why the same expression in a *document* is held to a higher standard
  (below).

Promotion is the one path from a session into the document, and it is **an
edit the person makes**: stopped at a breakpoint with `subtotal = 19.99`
visible, one action drafts a `<hick:capture>` with an `<hick:expect>` around
it and applies it through the ordinary edit path, undoable and reviewable like
anything they typed.

**Transport.** The socket already multiplexes by channel byte: `0x00` Yjs,
`0x01` runs, `0x02` LSP. Debugging is `0x03`, carrying DAP messages the way
`0x02` carries LSP ones, with a bridge that owns the session's lifetime and
rewrites paths between the client's scheme and the filesystem — the job
`LspBridge` does, written as a sibling rather than a generalisation, because
the two protocols' handshakes differ more than they look.

## 2. In a document: the same expressions, nobody stepping

```xml
<hick:exec container="lab">
python3 pricing.py
  <hick:capture at="pricing.py:14" of="subtotal, len(lines)" when="subtotal < 0" />
</hick:exec>
```

`at` is a location in a generated file, which is a location in the document.
`of` is a list of expressions. `when` is a breakpoint condition.

A condition wants `<` and `>`, and the no-escaping invariant means there is no
`&lt;` to fall back on — so this was checked before being specified: the
parser accepts both raw inside an attribute value (`title="a < b and c > d"`
round-trips today). A comparison reads as a comparison.

On a run the adapter sets that breakpoint, and each time it is hit the runner
issues **the same `evaluate` request the interactive pane issues**, records the
values, and continues. Nothing ever waits for a human. What comes back is
woven like any transcript:

| hit | `subtotal` | `len(lines)` |
|-----|-----------|--------------|
| 1   | `0`       | `3`          |
| 2   | `19.99`   | `3`          |

And because it is recorded output, `<hick:expect>` pins it. The document can
now assert something about a value **inside a function**, which no amount of
stdout checking reaches — re-derived on every run.

### Expression parity, and where it stops

**Yes: an expression that works in the interactive box works in a capture.**
Both are DAP `evaluate`, in a frame, in the debuggee's own language. There is
no second syntax to learn and no subset. That is the direct benefit of both
modes sharing an engine, rather than a capture being a `print` injected into
your code — which would have had subtly different scope and timing, and could
not evaluate in a chosen frame at all.

Three places the two modes are deliberately *not* the same, all because a
document is permanent and a session is not:

- **Side effects are a bug in a capture.** Evaluating `lines.pop()` in a
  session is your business; in a document it changes what the program does and
  therefore what the run records. Captures are sent with DAP's `watch`
  evaluation context, which adapters treat as repeatable, and the guidance is
  plain: a capture expression that mutates is a mistake the document will keep
  making.
- **Non-determinism is drift.** A captured value that changes every run trains
  people to ignore drift. Captures are named expressions rather than frame
  dumps, redacted by the volatile-output rules this repo already has, and
  ordered by hit index rather than wall-clock. A value that cannot be made
  deterministic belongs in a session, which writes nothing.
- **Hits are bounded.** A breakpoint in a hot loop evaluated ten thousand
  times is a slow run and an unreadable table. A capture takes a `max` with a
  small default, and the woven output says when it stopped early rather than
  silently truncating.

Where an adapter implements **log points** — a breakpoint that formats a
message and continues without stopping, which `debugpy` and `js-debug` both
do — the runner uses them instead: the same thing in one round trip rather
than three, with an identical observable result.

## 3. Over MCP: the same session, for an agent

An agent that can only read a failing document is guessing. An agent that can
stop at the failure and ask what a variable held is doing what a person does.

| tool | what it does |
|---|---|
| `debug_start` | Launch a cell under its adapter. Breakpoints (with conditions) given up front, in document coordinates. Returns a session id. |
| `debug_state` | Where it is stopped, the stack, and the selected frame's variables. |
| `debug_eval` | Evaluate an expression in a chosen frame. |
| `debug_step` | `over` \| `in` \| `out` \| `continue` \| `to_cursor` \| `drop_frame` (re-enter the current function). |
| `debug_stop` | End the session. |

Two rules that matter more than the surface:

- **A session is bounded.** It ends on an idle timeout and when the agent's
  turn ends. A debugger that outlives the conversation is a process holding a
  workdir open, and the first symptom is a later run failing for no visible
  reason.
- **The agent's session is as confined as a cell** — same sandbox, same
  capabilities, same denied network. An "attach to my process" tool that
  escaped the sandbox would undo the reason the sandbox exists.

## Adapters are installed exactly like language servers

`hick dap install python`, and the three rules that made `hick lsp install`
defensible:

- **Discovery first.** `debugpy` in the project's virtualenv, `js-debug` in
  `node_modules`, Delve on `PATH`, CodeLLDB in a VS Code extensions
  directory — whatever is already there wins.
- **Never automatic.** An install reaches the network and runs the package's
  setup scripts. It happens when a person types it.
- **Confined.** Into `.hick-cache/adapters/`, through the same sandbox, with
  the installer profile that keeps the toolchain visible and redirects `HOME`
  into the prefix.

And the lesson from `hick lsp install`, which shipped a broken package pair
because `typescript` on npm became the native port with no `tsserver.js`: an
adapter that is installed but cannot start must be **passed over at
discovery**, not spawned to fail later.

`<hick:needs>` extends here: a document with captures needs its adapter, and
saying so before the run beats a DAP error nobody can read.

## What the sandbox costs

This is the main risk, and it is worth stating before anyone starts.

A debug adapter controls another process: `ptrace` on Linux, `task_for_pid` on
macOS. The sandbox gives each cell its own PID namespace (`--unshare-all`), so
an adapter outside cannot see the process at all.

The resolution is that the adapter goes **inside**, launched as the cell's
entry point rather than attached from outside:

```
bwrap … -- debugpy --listen … -- python3 pricing.py
```

It then shares the namespace with its target and needs no extra privilege.
What that costs:

- The adapter must be **visible inside the sandbox**, so it is bound read-only
  into the cell's empty `$HOME`, joining the toolchain directories already
  there for the same reason `duckdb` needed to be.
- **`--unshare-pid` may still block ptrace between siblings** on hardened
  kernels — the same class of restriction that made `bwrap` fail outright on
  CI until detection learned to smoke-test it. If it does, the choice is a
  per-cell relaxation (stated in `describe()`, never silent) or refusing to
  debug under confinement. Refusing is the default: a sandbox that quietly
  weakens itself for a feature is worse than a feature that says it needs
  `HICKORY_EXECUTOR=local`.
- **Windows AppContainer** and debugging is a genuine unknown, on a path that
  has never been run at all. Assume it does not work until someone tries it.

### Proven, on Linux, 2026-08-14

The spike was run before anything else was built, and it passes.

A real DAP session — `initialize`, `launch`, a **verified** breakpoint,
`evaluate` in a frame across several hits with values that differ per
iteration, `next` / `stepIn` / `stepOut`, `continue` — driven over stdio
against `debugpy.adapter` running **inside `bwrap` with the cell's own
policy**: read-only root, private `/tmp`, empty `$HOME`, `--unshare-all` (so
no network), `--die-with-parent`. Identical results confined and unconfined.

Two things that spike established beyond the headline:

- **`debugpy` never needed `ptrace` at all.** It is an in-process debugger
  using Python's own tracing hooks, and the same is true of the Node and Ruby
  adapters. For those languages the PID-namespace worry was misplaced.
- **A native debugger works too.** `gdb` running a program under itself
  succeeds inside the sandbox exactly as it does outside — which is the
  `launch` mode CodeLLDB and Delve use. Only *attach to an already-running
  process* is restricted, and that is the host's `yama ptrace_scope=1`
  restricting it **identically outside the sandbox**. Since the design
  launches the cell under the adapter rather than attaching to it, this does
  not bite.

So no per-cell relaxation is needed and no `HICKORY_EXECUTOR=local` fallback:
on Linux, debugging works confined. macOS (Seatbelt) and Windows
(AppContainer) remain untested, and the Windows path has never been run at
all.

**The spike also found a live bug in the sandbox**, which is the argument for
running spikes before designs. The venv it created had its interpreter
symlinked into `~/.local/share/uv/python/…`, and the cell profile hid it —
`No such file or directory` for a Python that is plainly installed. The
profile bound `~/.local/bin` but not uv's store, and bound `~/.npm-global/bin`
without the `lib/node_modules` its launchers point at. Toolchain stores are
now bound at their **root** rather than their `bin`, because a `bin` entry in
a version manager is usually a symlink into a sibling directory and a symlink
whose target is hidden is a file that does not exist. `~/.cargo/bin` stays
narrow on purpose: its binaries are real files, and one level up sits the
credentials file for `cargo publish`.

## Order of work

1. ~~**Spike the sandbox question.**~~ **Done, and it passes** — see above.
   Debugging works inside the cell's own confinement on Linux, no relaxation
   needed. It also found and fixed a sandbox bug that had nothing to do with
   debugging.
2. **`hick-dap`**: adapter discovery, `hick dap install`, and a session API —
   launch, breakpoints, the four step verbs, `evaluate`, stack and variables.
   Driven by tests, no UI.
3. **`<hick:capture>`** on that session API: the non-interactive runner, the
   woven table, `<hick:expect>` over it, hit bounds and redaction.
4. **Channel `0x03` and the app's session UI**, including promotion. Where the
   isolation guarantee gets its tests.
5. **MCP tools**, once a human has driven the session API first.

Steps 2 and 4 are each worth roughly what this week's LSP work was; 3 and 5
are small once 2 exists.

One note on the ordering, since an earlier draft had it differently: there is
no cheap `print`-injection stopgap worth building first. A capture and the
interactive expression box must share one evaluator to be worth having, and
that means DAP comes first.

## What this is not

- **Not reverse debugging.** Out of scope by decision; see above. Drop frame
  (`restartFrame`) is in scope and covers most of what people want from it —
  re-running a frame rather than rewinding one.
- **Not a profiler.** Sampling and timing are a different tool with different
  determinism problems.
- **Not remote debugging.** Same machine, same rules as everything else here:
  no server we operate.
- **Not a replacement for `<hick:expect>`.** Most claims are about a cell's
  output and should stay that way. Captures are for claims that can only be
  made from inside a function.
