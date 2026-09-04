# Debugging The JVM

**Status: a design, investigated 2026-09-04, nothing built.** Java, Kotlin,
Scala and Groovy are IntelliJ's own subject and the largest hole in
`replacing-the-jetbrains-pack.md`. This is what it would actually take, and
what it must not claim before then.

The short answer to "do we need to write a custom plugin": **no.** The plugin
exists and is called `java-debug`. What does not exist is the *client* half,
and one part of that was built the same day for another reason.

## What java-debug actually is

Read from the project's own README rather than assumed:

- It is **not a standalone adapter.** There is no binary to put on PATH. It
  is an Eclipse plugin — `com.microsoft.java.debug.plugin` — that runs inside
  **eclipse.jdt.ls**, the Java language server.
- A client launches jdt.ls with the jar named in its `initializationOptions`:

  ```json
  { "initializationOptions": { "bundles": ["…/com.microsoft.java.debug.plugin-<v>.jar"] } }
  ```

- Then it sends an ordinary LSP `workspace/executeCommand`:

  ```json
  { "command": "vscode.java.startDebugSession" }
  ```

- **The response is a port number**, and the client connects to it and speaks
  ordinary DAP.

So the shape is: *language server → one command → a port → DAP over TCP.*

## Half of it is already built

`an-adapter-that-listens-is-connected-to.md` (2026-09-04) gave `hick-dap` a
TCP transport, because delve and js-debug both turned out to be servers. That
work is the same work: once a port is in hand, a Java session is an ordinary
`Adapter::connect_tcp`, and every existing thing — breakpoints in document
coordinates, the stack, `evaluate`, conditional breakpoints — applies without
change.

That is worth noticing rather than celebrating: it means the remaining
question is not about DAP at all.

## The one genuinely new concept

Every adapter hick knows is **found and spawned**. This one is **asked for**:
the port comes from a language server that must already be running, over a
protocol `hick-dap` knows nothing about.

**`hick-dap` must not depend on `hick-lsp`.** That inverts the layering, and
it would make every debug session in every language capable of dragging a
language server behind it. The seam belongs where both already meet:
`hickory-cli` owns `LspHub` (one language-server session per workspace, since
`one-language-server-per-workspace.md`) *and* `DebugSessions`. So:

- `hick_dap` gains a transport that means **"connect to this port; somebody
  else obtained it"** — a small widening of what already exists, not a new
  subsystem.
- `hickory-cli` supplies the port by asking the hub's Java session to execute
  the command.

Said as a rule: **an adapter may be found, or it may be asked for, and
whoever can ask is the one that already holds the conversation.**

## The hard part, which is not the protocol

A document's Java is generated into a **scratch directory**. jdt.ls does not
debug files; it debugs a project it has **imported**, and import is the slow,
stateful thing that makes a JVM IDE feel like one. Two sub-questions, and the
first must be answered before any of this is worth starting:

1. **Can jdt.ls import a scratch tree with no build file?** It has a mode for
   standalone Java files outside any project. If that mode supports launching,
   a document generating one `Main.java` works. If it does not, a document
   must generate a `pom.xml` or a `build.gradle` — and that is fine, and is
   exactly what `owning-what-a-scaffolder-wrote` already expects of C#, but it
   must be **said** rather than discovered at launch. *This is unverified and
   is the first thing to measure.*
2. **Import is not instant.** A first import of a real project is seconds to
   minutes. The `Build` step already gives a place to put that honestly — a
   build is watched in a terminal — so "importing the project" belongs there,
   visible, rather than behind a spinner.

## The licence question, decided

`java-debug` is **EPL-1.0**. The workspace rule is: *a dependency whose
licence is GPL or otherwise copyleft cannot be **linked** into this product.*
EPL is weak copyleft, and the operative word is *linked*.

Nothing here is linked. The jar is fetched onto the user's machine at install
time, unmodified, and loaded by a **separate JVM process** that hick spawns —
the same relationship hick already has with netcoredbg and codelldb, and the
same one it already has with **eclipse.jdt.ls itself**, which is EPL-2.0 and
which `hick-lsp` has discovered and run since before this question was asked.
Refusing java-debug while running jdt.ls would be incoherent.

Recorded as a decision rather than left implicit, because the heading in
`AGENTS.md` has already had to be amended once when a licence question
surfaced (MIT-only → No copyleft, for Apache-2.0 `scip`). This is the same
kind of moment and does not need the same amendment: the rule as written
already answers it.

## Kotlin, Scala and Groovy are three more investigations, not this one

They share a runtime and nothing else that matters here.

- **Kotlin** has `kotlin-debug-adapter`, a separate project that is a real
  standalone adapter. If it speaks stdio it needs no new concept at all.
- **Scala** debugs through **Metals**, which exposes DAP via BSP — a third
  arrangement again, and closer to the java-debug shape than to the Kotlin
  one.
- **Groovy** has no maintained adapter this investigation found.

Each needs its own measurement. Nothing above should be read as covering
them, and `hick lang` must go on reporting all four as Bronze until one of
them has been driven to a stop — the rule that was just re-proved by four
languages at once in
`an-adapter-that-listens-is-connected-to.md`.

## The order, if this is picked up

1. Verify (1) above: start jdt.ls by hand on a scratch tree, with the bundle,
   and see whether `vscode.java.startDebugSession` answers with a port. One
   afternoon, and it decides everything after it.
2. Widen the transport to "a port somebody else obtained".
3. Wire the seam in `hickory-cli`, where the hub already lives.
4. `hick lsp install java` fetches jdt.ls **and** the bundle, pinned, through
   the same confined archive installer the other adapters use.
5. A live test that stops on a document line and reads a value out of the
   frame. Until that passes, Java is Bronze.
