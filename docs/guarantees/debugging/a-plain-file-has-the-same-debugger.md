# A Plain File Has The Same Debugger A Document Has

Given a folder open in the app and a file in it that is not a `.hick`
document — `src/main.rs`, `tools/app.py`, `index.ts`, `Program.cs` — in a
language hick can debug, when it is opened in a pane, then the pane has the
breakpoint gutter, the Debug button, and the debugger's strip a document's
block has; a breakpoint on line *n* is a breakpoint on line *n* of the file;
the program runs **in its own project**, built by that project's own tools
where the language needs a build; and while paused the pane shows the paused
line, the inline values, the stack, the watches, the hover value and the
inline eval exactly as a document's editor does. A frame in another file of
the folder is opened in that file's tab at the adapter's line.

A file in a language hick cannot debug gets neither the gutter nor the
button: a gutter that takes a dot it can never bind is a promise the pane
cannot keep.

The app could debug a document's generated Python and could not put a
breakpoint in the `main.rs` of the repository it was open on. For someone
replacing their IDE with this one, that is the daily loop with a hole in it.

Four properties hold it up, and each is the same move the language server
made for plain files:

1. **The file is its own program.** `hick_dap::Mapping::identity` maps every
   line to itself, so every caller that translates through a `Mapping` —
   breakpoints, frames, run-to, jump — works unchanged. A frame in another
   file keeps its `source` and a new `source_line`, so what this pane cannot
   show it can still open.
2. **The build is the project's own.** `hick_dap::build_plain` finds the
   nearest project file above the file (never above the folder the app
   opened), builds in place, and asks cargo where its target directory is
   rather than redirecting it — the debugger shares incremental state with
   the person's own `cargo build` instead of building a second copy of the
   repository under `.hick-cache/`. Nothing is invented: a `.cs` with no
   `.csproj` above it is refused by name, with the fix named.
3. **It runs where the person's tools would run it.** `Registry::start_plain`
   launches with the nearest project directory as the working directory and
   no scratch copy. A plain file has nothing woven to protect and no
   transcript to keep honest; it is the person's program in the person's
   checkout, and it reads and writes what it would under `cargo run`.
4. **The pane wires the whole layer over the workspace socket, and the
   session lives above the pane.** `PlainFilePane` mounts `debugEditor`
   with the file's language as the whole-file language, so every non-blank
   line can hold a breakpoint; `useWorkspaceDebugger` shares one
   `DebugClient` over the workspace connection (which now carries the debug
   channel as well as the language one); and because every plain-file pane
   shares that connection, the server names the file on `started`, `build`
   and a failed `start`, and each pane keeps only the events for its own
   file or its own session. The session itself is held by a workspace-level
   **host** (`debug/plainDebugHosts.tsx`), one per path, never unmounted —
   because only a pane's active tab is rendered, and a session held in the
   pane's own state died the moment stepping into a callee fronted the
   callee's tab. Found in a real browser on 2026-09-03; a document never
   had this problem because its session lives in the workspace registry.

## Boundary

A document is still debugged in a scratch copy; nothing here changes
`debugging-never-writes-to-the-project.md`, which now says so in its own
boundary. Only one file is debugged at a time per pane. When the program
stops in *another* file of the folder, that file's tab is opened and its
pane draws the paused line (published through `lib/pausedElsewhere.ts`,
cleared by the owning session when it moves on); values, watches and the
strip stay with the owning pane. Which program a multi-binary package launches is
answered the way it is for a document — the artifact named after the
package — and a package with several and no match is a refusal that lists
them, not a guess.

---

Last LLM verification:
- Date: 2026-09-03
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hick-dap/src/session.rs` — `Mapping::identity`,
  `Frame::source_line`; `crates/hick-dap/src/build.rs` — `build_plain`,
  `nearest_matching`, `cargo_target_dir`, `BuildEnv::Own`;
  `crates/hickory-cli/src/debug_sessions.rs` — `Registry::start_plain`,
  `is_document`, `project_dir_of`; `crates/hickory-cli/src/serve/debug_bridge.rs`
  — the `Start` arm branches on `is_document`, `Started`/`Build`/`Failed`
  carry `doc`, `relative_source`; `crates/hickory-cli/src/serve/socket.rs`
  — `forward_debug`, shared by the document socket and
  `run_workspace_socket`. In the app: `apps/web/src/debug/useDebugger.ts`
  (`useWorkspaceDebugger`, `useDebuggerOver`, `eventIsOurs`),
  `apps/web/src/debug/cmDebug.ts` (`wholeFileLanguage`, `debugStateEffects`,
  the `lineNumbers` option), `apps/web/src/components/PlainFilePane.tsx`
  (the gutter, the strip, the Debug button, `selectFrameAndReveal` opening
  another file through `openLocation`).
- Test coverage: `crates/hickory-cli/tests/debug_desktop.rs::a_plain_file_debugs_as_itself_over_the_workspace_socket`
  (a folder with no document; breakpoint, stop, values, step into the
  neighbouring file whose frame is named root-relative with its line, the
  file unchanged afterwards) and
  `a_plain_rust_file_builds_in_its_own_project_and_stops_on_its_own_line`
  (a cargo workspace member: no scratch copy, the binary in the
  workspace's own `target/`, a stop on the file's own line under codelldb);
  `crates/hick-dap/src/session.rs::a_plain_file_maps_every_line_to_itself`;
  `crates/hick-dap/src/build.rs::the_nearest_project_file_above_a_plain_file_wins`,
  `a_plain_interpreted_file_is_its_own_program`,
  `a_plain_compiled_file_with_no_project_is_refused_by_name`;
  `crates/hickory-cli/src/debug_sessions.rs::a_plain_file_runs_in_the_nearest_project_above_it`;
  `apps/web/src/debug/useDebugger.test.ts` ("two panes on one socket");
  `apps/web/src/debug/cmDebug.test.ts` ("a plain file's breakpoints");
  `apps/web/src/components/PlainFilePane.test.tsx` ("a plain file and the
  debugger", rendered beside `PlainDebugHosts`);
  `apps/web/src/debug/DebugStrip.test.tsx` (a frame in another file of the
  folder is named by file and line, not "external").
- Caveats: the live tests (Python, Rust, and C# —
  `a_plain_csharp_file_builds_its_own_project_in_place_and_stops_on_its_own_line`,
  `dotnet build` in the file's own project, the assembly under its own
  `bin/`) skip loudly when the adapter or toolchain is missing, so a machine
  without them verifies the shape and not the running program. The whole
  loop — breakpoint, Debug, paused line with inline values, step into the
  neighbouring file, its tab opening with the paused line, the stack naming
  it, Stop — was driven in a real browser (Playwright over the Vite dev
  server and the engine) on 2026-09-03; the automated coverage of the
  browser side is jsdom.
