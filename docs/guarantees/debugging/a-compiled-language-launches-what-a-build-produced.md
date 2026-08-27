# A Compiled Language Launches What A Build Produced

Given a document that generates source in a **compiled** language, when a
debugger is started over it, then what the adapter launches is the artifact a
build produced — not the file the document generated:

- `entry_point` still names the **source**, and `adapter_for` still reads the
  language from it. Both are unchanged, because the source is what has a
  language. What that source *produces* is what gets launched.
- For Python, Node and Go the build step returns the path it was given. Go is
  the interesting one and it stays `None`: it is compiled, and delve compiles
  it on the way past.
- For C#, `dotnet build` runs **in the project file's own directory**, found
  at whatever depth the document put it, and the program is the assembly it
  wrote.
- **A document that generates no project file is refused, by name.** No
  project file is invented: a plausible `.csproj` written on somebody's behalf
  produces a program that is not theirs. The refusal says C# is compiled, says
  what is missing, and says what to generate.
- **Nothing is fetched silently.** Package restore is announced before it can
  happen, naming the directory packages land in and saying the first build of
  a project may reach the network.
- **A build is watched, not summarised.** Its output reaches the client as
  terminal events and is drawn by the read-only watching terminal
  (`a-watched-cell-shows-what-the-terminal-showed.md`). It arrives *before*
  the failure on the failing path, which is the case it exists for: a build
  that fails says why in the compiler's own words, and "build failed" throws
  all of that away.
- **The build is not evidence.** Nothing here is recorded under a cache key,
  woven, or compared. The transcript is the record; the terminal is the run
  happening, and the surface that shows it says so.

**The editor has to name C# too, in its own list.** The gutter decides whether
a line can hold a breakpoint before the server is asked — `breakpointLine`
consults `apps/web/src/debug/languages.ts`, which is kept in step with
`hick_dap::discovery::candidates` by hand. That list is the last place C# has
to be named, and while it was missing the whole path above was unreachable
from the app: a `.cs` file block offered no ghost dot, the breakpoint gutter
rendered no cells at all, and a click in the gutter did nothing whatever —
not even the refusal that names the language.

One thing this deliberately does **not** guarantee: nothing here is
sandboxed — `Adapter::spawn` already starts the adapter directly, and the
isolation on this path is the scratch copy, not the executor that confines
cells; a build step is no *less* confined than the debuggee it feeds, but it
does run a build tool the document's own content chose.

> **Amended 2026-08-27.** This document used to close by saying there is no
> `hick dap install csharp`. There is one now — netcoredbg still ships as
> release archives rather than as one installable command, so the catalogue
> grew an archive shape instead
> ([an-adapter-that-ships-as-an-archive-installs-like-any-other](an-adapter-that-ships-as-an-archive-installs-like-any-other.md)).
> Discovery still prefers a netcoredbg the user installed themselves.

---

Last LLM verification:
- Date: 2026-08-27
- Reviewer: Claude (Opus 5)
- Result: verified for the build and for the gutter; the launch is
  **unverified** — see caveats
- Evidence:
  - `apps/web/src/debug/languages.ts` — `DEBUGGABLE_LANGUAGES` gains
    `"csharp"` (2026-08-27). Confirmed in the running app against
    `.dev/project/scaffolding.hick`: the breakpoint gutter went from **one**
    element (CodeMirror's width spacer, and nothing else) to a ghost on each
    line of the `app/Program.cs` block, and a click set a real dot on
    `Console.WriteLine`. The `app/app.csproj` block beside it is XML and
    correctly offers nothing.
  - `crates/hick-dap/src/build.rs` — the `Build` table, `build_for`, and
    `build`. `crates/hick-dap/src/discovery.rs` gains netcoredbg
    (`--interpreter=vscode`) and `csharp` in `known_languages()`, **in this
    same change**, which is the spec's fourth refusal: on its own that entry
    would make `language_of` offer C#, pick `Program.cs`, and fail at launch.
  - `crates/hick-lsp/src/lang_detect.rs` gains `"cs" => Some("csharp")`.
    **This was a real gap, not a new requirement:** that table is the only
    thing routing a generated file to a language, for the language server and
    the debugger both, and it had no `cs` row — so `hick lsp install csharp`
    had been fetching a server that could never be reached, because every
    `.cs` virtual file got `language_id: None` and was skipped in
    `backend.rs` before discovery was consulted.
  - Call sites: `crates/hickory-cli/src/debug_sessions.rs` (interactive),
    `crates/hick-dap/src/capture.rs` (`<hick:capture>`, output to the log),
    `crates/hickory-cli/src/mcp.rs` (agent, output attached to the failure).
  - Client: `Response::Build` in `serve/debug_bridge.rs` carries
    `TranscriptEvent`-shaped events; `apps/web/src/debug/DebugStrip.tsx`
    draws them with `WatchingTerminal`.
- Test coverage:
  - `apps/web/src/debug/cmDebug.test.ts` — "takes a line of C#, which
    netcoredbg debugs" asserts `breakpointLine` accepts a `.cs` block's code
    line from the path's extension alone, and still refuses the `.csproj`
    beside it.
  - `crates/hick-dap/src/build.rs` unit tests: interpreted languages are
    handed back their own path, the no-project refusal says all three things
    *and* asserts no `.csproj` was written, the wildcard matches within one
    path component only, the glob does not wander into `obj/`, the assembly
    named after the project wins over its dependencies, and several
    candidates with no stem match is a refusal that lists them.
  - `crates/hick-dap/src/build.rs::csharp` — **a real `dotnet build`**
    (10.0.111 on this machine): a project at `app/` builds, the returned
    program is `app/bin/Debug/net10.0/app.dll` and exists, the restore notice
    came before the build, and a syntactically broken `Program.cs` surfaces
    `error CS…` in the build's own words. Gated on `dotnet` being present and
    **saying so when it skips**, because a test that returns before its first
    assertion is a green suite covering nothing — the shape that already bit
    `crates/hick-term/tests/full_screen_apps.rs`.
  - `crates/hick-dap/tests/csharp_document.rs` — the same path from a real
    document: weave, walk past the generated `.csproj` to the `.cs`, build,
    and assert the launched program is *not* the generated file.
  - `crates/hickory-cli/src/serve/debug_bridge.rs` — a failed start still
    delivers whatever the build said, before the failure, and a build line is
    shaped like a transcript event.
  - `crates/hickory-cli/src/dap_install.rs` — the drift check between the
    catalogue and the advice. **It caught a live bug the moment it existed:**
    the "no adapter" message had been telling people to run
    `hick dap install go` (and `rust`, `c`, `cpp`, `ruby`), none of which the
    catalogue can install. Each now names the adapter and that ecosystem's
    own way to get it.
  - `apps/web/src/debug/DebugStrip.test.tsx` — the build terminal appears
    while starting and stays when the start failed, disappears once the
    program is running, is absent for a language that needs no build, and
    says on screen that it is not a transcript.
- Verified by running: `hick dap list` now names the languages hick can
  **debug** as well as the ones it can **install**, because the error a person
  arrives from says "`hick dap list` names them" about the first set and the
  command used to print only the second — someone with a Go file was sent to
  a page that did not mention Go.
- Caveat requiring review:
  - ~~No C# debug session has ever been started.~~ **Resolved 2026-08-27, at
    the user's request.** netcoredbg 3.2.0-1092 was fetched and unpacked
    under bubblewrap (only its prefix writable) and a real session run:
    `crates/hick-dap/tests/live_session_csharp.rs` builds a C# document,
    launches the assembly, stops on a breakpoint set on a **document** line,
    reports a top frame named `LineTotal` on that same document line, and
    reads `quantity` as `3` out of a live .NET frame. The pdb half of the
    mapping is therefore observed rather than assumed. The test skips loudly
    and separately for a missing SDK and a missing adapter, and the skip path
    was exercised too.
  - ~~netcoredbg does not verify a breakpoint at set time…~~ **Fixed
    2026-08-27**, and it is now its own guarantee:
    `a-breakpoint-that-is-not-bound-yet-is-not-refused.md`.
  - **The artifact glob is a guess disambiguated by a convention.**
    `bin/Debug/*/*.dll` can match a project's dependencies as well as its own
    assembly, so the project file's stem wins — which is .NET's default
    assembly name and not a rule. A project that sets `<AssemblyName>` and
    has dependencies gets a refusal listing the candidates rather than a
    wrong launch, which is the right failure but is still a failure.
  - **Where the build command comes from is the spec's open question, and it
    is not answered here.** The built-in recipe is implemented; the
    document's own `builds=` attribute — which would run the build the way
    cells run, confined, and remove the glob entirely — is not. The spec
    calls the first shippable and the second right.
  - **The table is scaffolding and this does not extend it.** The shape
    underneath is that a compile is a cell whose output is a file, and the
    debugger should consume the DAG rather than bypass it. The spec names the
    signal to stop: the moment a second compiled language needs a fourth
    field.
  - **The build output is delivered in one batch, not streamed.** `handle`
    returns a `Vec<Response>`, so a long build shows nothing until it
    finishes and then shows everything. The terminal is live-capable (it
    appends), but nothing is currently feeding it live. Streaming needs the
    frame sender threaded into `handle`, which is a change to that function's
    shape rather than to this one.
  - **Windows is untested.** `dotnet` is cross-platform and nothing here is
    POSIX-specific, but no part of this has run there.
