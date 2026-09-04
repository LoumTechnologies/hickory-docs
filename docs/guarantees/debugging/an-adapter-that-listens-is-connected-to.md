# An Adapter That Listens Is Connected To

Given a debug adapter that is a server rather than a filter, when a session
starts, then hick spawns it with a free port substituted for `{port}` in its
arguments, connects to it over TCP, and speaks the same DAP over that socket
as it does over a pipe. Discovery says which transport an adapter wants, and
carries any launch keys that adapter requires so no caller has to know them.

A language is listed as debuggable **only when an adapter for it can actually
be driven to a stop**. Being able to find or install one is not enough.

## Why

`Adapter::spawn` was stdio-only, and that was invisible because the three
adapters with live tests — debugpy, netcoredbg, codelldb — all speak stdio.
Every language *without* a live test turned out to be broken, and the
correlation was exact. Measured on 2026-09-04:

- **Go.** `dlv dap` is, in delve's own help, "a headless TCP server
  communicating via Debug Adaptor Protocol". Handed a pipe it says nothing at
  all; the session hung with no error, forever.
- **JavaScript and TypeScript.** `npm install @vscode/js-debug` returns 404
  and always has — the package is not published to npm — so
  `hick dap install typescript` could never have worked on any machine. The
  adapter also prints "Debug server listening at ::1:8123" and never reads
  stdin.
- **Ruby.** `rdbg --open target.rb` opens a UNIX domain socket, and runs the
  program itself: the program is chosen when the adapter starts, so there is
  nothing for a `launch` request to say.
- **C and C++.** A different fault, found by the same absent test: they fell
  through `build_for` to `Build::None`, whose meaning is "the generated file
  IS the program". The debugger was handed `main.c`.

All of them were reported as Silver by `hick lang`.

## What this bought, and what it did not

Go and C/C++ work now and have live tests that stop on a document line and
read a value out of the real frame. C and C++ got a `Build::Compile` step:
there is no universal C project file — a Makefile, a CMakeLists.txt and a
bare `cc main.c` are all normal — and a one-file program is a complete
program, which is exactly what a literate document generates. Inventing a
CMakeLists.txt would be the same mistake as writing somebody a `.csproj`.

**JavaScript, TypeScript and Ruby are now reported as NOT debuggable**, which
is the change that matters most. js-debug is reachable — the archive
installer and the TCP transport both work, and were built — and it still
cannot be driven, because it is a **multi-session** adapter: it answers
`launch`, marks the breakpoint `provisionalBreakpoint`, and then sends a
`startDebugging` reverse request carrying a `__pendingTargetId`, expecting
the client to open a second connection and run a whole second session where
the breakpoints actually bind. `Session` owns one adapter and one event
stream, so that is a session tree, not a flag. Ruby needs a third adapter
shape again.

Saying nothing is better than offering a debugger that attaches to a program
and never stops in it.

---

Last LLM verification:
- Date: 2026-09-04
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hick-dap/src/adapter.rs` — `Transport`,
  `PORT_PLACEHOLDER`, `free_port`, `Adapter::start`/`connect_tcp`, and `wire`
  (the half both transports share); `crates/hick-dap/src/discovery.rs` — the
  `LANGUAGES` table, which is now the only statement of both which languages
  are debuggable and what debugs them, plus `Candidate::transport` and
  `Candidate::launch`; `crates/hick-dap/src/build.rs` — `Build::Compile`,
  `first_on_path`, `compile_one`; `crates/hick-dap/src/session.rs` —
  `Launch::transport` and `Launch::adapter_extra`, merged under the caller's
  own `extra`.
- Test coverage: `crates/hick-dap/tests/live_session_go.rs` and
  `live_session_c.rs`, both of which stop on a document line and read an
  argument from the real frame, beside the existing Python, C# and Rust ones
  — all five pass on this machine;
  `discovery::tests::known_languages_and_candidates_agree`, which now asserts
  **both** directions, including that JavaScript, TypeScript, the React ids
  and Ruby are NOT advertised while nothing can drive their adapters.
- Caveat requiring LLM review: the live tests are skipped loudly when a
  toolchain is missing. That weakness was not theoretical — CI installed no
  debug adapter at all, `.hick-cache` is gitignored, and every live debug
  suite skipped while the job went green. `crates/hick-dap/tests/debug_coverage.rs`
  now fails when nothing was debuggable, and CI installs the adapters before
  the tests. C# is still uncovered there, because netcoredbg needs the .NET
  SDK that the runner deletes for disk space.
  `{port}` is claimed and released before the adapter binds it, which is a
  race no DAP client can close (adapters take a port number, not a listening
  socket) and is named in the code rather than hidden. Only `linux-x86_64`
  has been exercised.
