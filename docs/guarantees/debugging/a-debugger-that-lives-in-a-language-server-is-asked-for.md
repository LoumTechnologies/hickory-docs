# A Debugger That Lives In A Language Server Is Asked For

Given Java, when a debug session starts, then hick starts **eclipse.jdt.ls**
with the `java-debug` plugin named in its `initializationOptions.bundles`,
waits for the server to report `ServiceReady`, sends
`workspace/executeCommand` for `vscode.java.startDebugSession`, and connects
to the **port** in the reply. From there it is ordinary DAP: breakpoints in
document coordinates, the stack, values, stepping.

The launch arguments are the server's too. java-debug does not take a
`program`; it takes a `mainClass` and a `classPaths`, and only the language
server knows either — they come from `vscode.java.resolveMainClass` and
`java.project.getClasspaths`.

Java is reported as debuggable **only when both halves are installed**
(`hick lsp install java` and `hick dap install java`), because the debugger
runs inside the server and one without the other can debug nothing.

## Why the seam is where it is

`hick-dap` does not depend on `hick-lsp`, and must not: that inverts the
layering, and would let a debug session in any language drag a language
server behind it. So `hick-dap` gained one thing — `Transport::Attached(port)`,
meaning *somebody else is listening; spawn nothing* — and the crate that owns
both ends, `hickory-cli`, does the asking.

The **filesystem** question is different from the **protocol** question, and
they are split accordingly. Whether the two halves are installed is an
ordinary path lookup, and discovery has to answer it to report capability
honestly, so it lives in `hick_dap::java` at the layer that reports
capability. `hickory_cli::java_debug` — which can speak LSP — uses the same
answer rather than a second copy of those paths.

The debug session gets its **own** short-lived jdt.ls, rooted at the scratch
directory the document wove into. The editor's session is rooted at the folder
the app opened, which is not where a document's code is, and entangling the
two lifetimes would let a debug session end the editor's intelligence.

## What was measured, and what it settled

`debugging-the-jvm.md` named one open question and said it decided everything
after it: **will jdt.ls import a directory with no build file?** Measured on
2026-09-04 by hand, before any of this was built:

- A directory holding one `Main.java` and no `pom.xml`, no `build.gradle`,
  nothing — jdt.ls creates an "invisible project", and
  `vscode.java.startDebugSession` answers with a port.
- It compiles with its own bundled compiler. **No `javac` on the machine is
  required**; a JRE is enough to run the server and the program.
- A breakpoint set over that port came back `verified: true`, and the program
  stopped on it.

So a document does not have to generate a build file, which is the answer
that made this worth building. A cold import of a one-file project took under
eight seconds end to end in the live test.

## On the licence

`java-debug` is EPL-1.0 and eclipse.jdt.ls is EPL-2.0. The workspace rule
forbids **linking** copyleft, and nothing here is linked: both are jars,
fetched onto the user's machine at install time, unmodified, loaded by a
separate JVM that hick spawns. It is the same relationship this product
already had with jdt.ls before any of this, through `hick lsp`. Recorded as a
decision in `debugging-the-jvm.md`.

jdt.ls is fetched from the `redhat.java` extension rather than from
download.eclipse.org, and that is about pinning rather than preference:
eclipse.jdt.ls publishes `jdt-language-server-latest.tar.gz`, whose URL is
stable and whose bytes change daily, and its milestone directories serve no
listing to resolve a version from. An archive installer pins bytes.

---

Last LLM verification:
- Date: 2026-09-04
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-cli/src/java_debug.rs` — `prepare`, the minimal
  LSP client, `wait_ready` (which waits for `ServiceReady`, not `Started`:
  the latter arrives before the import and a command sent then answers about
  a project that does not exist), `main_class` and `class_paths`;
  `crates/hick-dap/src/java.rs` — the shared locator, including that the
  launcher is `org.eclipse.equinox.launcher_*` and not one of the
  platform fragments beside it; `crates/hick-dap/src/adapter.rs` —
  `Transport::Attached` and `Adapter::attach`, which owns no process;
  `crates/hick-dap/src/discovery.rs` — `Recipe::LanguageServerHosted` and
  `Discovered::hosted`; both install catalogues, pinned.
- Test coverage: `crates/hickory-cli/tests/live_session_java.rs` — stops on
  the document line, asserts the frame is `lineTotal`, and reads
  `quantity == 3` out of the real frame; it also asserts the adapter is
  `hosted` with an empty command, so it cannot quietly pass down a spawned
  path. `hick_dap::java::tests` covers the launcher-versus-fragment trap and
  that a machine with neither half installed answers `None` rather than
  guessing. Eight languages now debug live together: Python, C, C#, Go,
  Java, JavaScript, Rust, TypeScript.
- Caveat requiring LLM review: only a **launch** is supported, not attach,
  and only the main class the file names. A cold jdt.ls costs seconds and a
  JVM's worth of memory per debug session, and nothing reuses one between
  sessions of the same project beyond the `-data` workspace cache. Kotlin,
  Scala and Groovy share the JVM and none of this: they are separate
  adapters and separate investigations, and all three remain Bronze.
