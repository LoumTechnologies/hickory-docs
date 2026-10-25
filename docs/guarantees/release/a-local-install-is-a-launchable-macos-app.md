# A Local Install Is a Launchable macOS App

Given a checkout on an Apple Silicon Mac with the desktop build prerequisites
and write access to `/Applications`, when `just local-install` succeeds, then
`/Applications/Hickory Docs.app` contains a native release build with the UI
embedded and is registered with macOS as **Hickory Docs**. It can be launched
from Applications without a running development server or this checkout. If
the per-user LaunchServices service is unavailable, the installer warns but
still succeeds with the valid app in Applications; macOS registers it when the
service returns.

The bundle declares `.md` files as Markdown documents with the Editor role.
After registration, the installer checks the current user’s Markdown default
through AppKit. If needed, it makes one asynchronous default-app request and
waits for macOS’s consent prompt to finish before reading back the result.
Before requesting the default, the helper unregisters other copies with the
same bundle identifier within this checkout's Cargo target directory; those
files are preserved. An existing default pointing at the installed app needs
no request or prompt. The helper allows up to five seconds for readback after
completion and never repeats the consent request. It does not change the
default for ordinary `.txt` files.

A default-editor failure does not fail an already installed and registered app.
It warns that the association could not be confirmed and gives Finder's
**Get Info → Open with → Change All…** recovery steps. Logs include the content
type, requested and current app paths, registered copies, running Hickory apps,
and any returned macOS error domain/code. Build and installation failures remain
fatal and identify the failed phase. The recipe saves output to
`.dev/local-install.log`, replaced on each run after prerequisites pass.

Finder’s file-open URLs are handled both at startup and while the app is
running. A startup request selects the file before the editor loads; later
requests open a new window/session through the same explicit-file launch path.

The installed bundle also carries the experimental Python agent as a
self-contained runtime under `Contents/Resources/python-agent`, staged by
`scripts/build-python-agent-runtime.sh`. The app discovers it from its own
location, so the agent needs no path in Settings; an explicitly configured
runtime still wins. See
`../agent/a-desktop-app-bundles-its-python-agent.md`.

A failed build or staging copy leaves an existing installed app untouched.
Re-running the recipe replaces the installed bundle. User documents and
settings are not part of that replacement.

After successful installation and registration, the recipe attempts to
unregister and removes the build-output `.app` from the checkout so macOS
cannot discover that copy as a second launcher entry. If macOS refuses to
unregister the build copy, cleanup warns and still removes it; this does not
fail the installed app's registration. A failed installation keeps the build
copy available.

The main window starts with an unsaved, untitled literate document explaining
Hickory Docs, with Files open by default when a folder is open and Agent
available on demand. Startup ignores the
previous pane arrangement and Welcome preference. See
`../authoring/the-app-starts-as-a-lightweight-editor.md`.

---

Last LLM verification:

- Bundled Python agent (2026-10-10, Codex): the recipe now runs
  `just stage-python-agent` before `cargo tauri build` and passes the generated
  `bundle.macOS.files` config. A debug `.app` built the same way contained
  `Contents/Resources/python-agent`, its `bin/python` symlink was preserved,
  `codesign --verify --deep --strict` passed, and the worker imported from the
  bundle. Starting that bundle's own `hick up` engine returned
  `available: true` from `/api/agents/python`. The `.dmg` path
  (`scripts/dist-desktop.sh`) shares the staging step; a disk image was not
  rebuilt for this entry. Full evidence:
  `../agent/a-desktop-app-bundles-its-python-agent.md`.
- Duplicate-default fix (2026-10-08, Codex): the user's report showed a
  completed build and app replacement followed by a default-editor readback
  failure. Independent AppKit inspection resolved Markdown to
  `apps/desktop/src-tauri/target/debug/bundle/macos/Hickory Docs.app`, with no
  Hickory process running. Both that copy and `/Applications/Hickory Docs.app`
  were registered under the same bundle identifier. The earlier cleanup only
  covered the release bundle; it assumed no other checkout build competed.
  `scripts/default-markdown-editor.rs::BRIDGE` now unregisters registered
  copies beneath the supplied Cargo target root without deleting them.
  Running the helper changed the resolved default to the installed app without
  a new consent request. `justfile::local-install` now treats association failure
  as a warning after successful installation, as requested by the user, and
  writes phase diagnostics and output to `.dev/local-install.log`.
  Executed checks: full `just local-install` exited 0; strict bundle signature
  verification passed within the recipe; an independent AppKit query for an
  actual temporary `.md` file resolved to `/Applications/Hickory Docs.app`;
  `.txt` still resolved to TextEdit. An isolated execution of the recipe's final
  section with a helper forced to exit 1 retained exit 0 and printed the warning,
  Finder recovery steps, and launch message. Helper compilation with warnings
  denied, recipe `bash -n`, file-length check, and scoped `git diff --check`
  passed. The full diff check found pre-existing trailing whitespace in
  `apps/web/src/content/startup.md`, left untouched.
  Coverage gaps: the new five-second asynchronous readback timeout and native
  error/consent paths were reviewed and compiled but not forced on macOS;
  running-app diagnostics were not exercised with a live Hickory process.
  No human judgment of these changes has been recorded.
  Inspected implementation SHA-256 (evidence becomes stale when these change):
  `justfile`: `0a5d59b2e167da1770dd58dd2c84a614a97ae0f9e2bdb6f055dce7dc0f3126c8`.
  `scripts/default-markdown-editor.rs`: `7e88f0fefa8a164c49991c8cda5872f04f940ff3b75a9dbece47bf1e08536970`.

- Markdown defaults (2026-10-03, Codex): `just local-install` completed
  after the user accepted macOS’s switch confirmation. The installer helper
  now uses AppKit’s asynchronous default-app API, waits for completion, reads
  back the result, and skips an already-selected app. A second installation
  completed with “already the default” and no new prompt. An independent
  `NSWorkspace.urlForApplication(toOpen:)` lookup resolved a temporary `.md`
  to `/Applications/Hickory Docs.app`. Launching that file via `open -n`
  (without naming an application) started the installed app; its real HTTP
  `/api/files` response named the temporary file’s containing folder and
  document. This checks native dispatch and workspace selection, not the
  rendered editor tab or later requests to an already-running process.
  `just check-desktop`, helper compilation with warnings denied,
  `just check-file-length`, and `git diff --check` passed.

- Cleanup fix (2026-10-03, Codex): `justfile::local-install` keeps installed
  app registration mandatory but treats build-copy unregistration as best
  effort. `just local-install` completed with exit code 0 on this Mac despite
  reproducing LaunchServices' `-10814` during cleanup; the build copy was
  removed and the installed bundle passed strict signature verification.
  A temporary shell check also forced unregister failure and confirmed the
  warning, removal, and successful exit. Recipe syntax and `git diff --check`
  passed. This verifies installation cleanup, not the startup UI.
- Startup review (2026-10-02, Codex): the introduction replaces the Welcome
  startup surface. `App.test.tsx` verifies the real editor and menu commands;
  the native launch observation below describes the earlier build.
- Cleanup review (2026-10-01, Codex): the build copy is unregistered and
  removed only after replacement and registration of the installed app
  succeed. `just --dry-run local-install` passed `bash -n`, and
  `git diff --check` passed. The revised installation was not run.
- Date: 2026-09-29
- Reviewer: Codex
- Result: verified on this Apple Silicon Mac.
- Evidence: `justfile::local-install` invokes Tauri's release build for
  `aarch64-apple-darwin` with `--bundles app`, stages via `ditto`, replaces the
  installed bundle only after staging succeeds, and calls `lsregister`.
  It uses an explicit ad-hoc signing identity and verifies the bundle before
  replacement, following Jobsearch's `family-build` convention.
  `apps/desktop/src-tauri/src/server.rs::Ui` embeds the built frontend;
  `apps/desktop/src-tauri/src/lib.rs::launch` chooses the remembered or default
  workspace. `apps/web/src/views/WorkspaceView.tsx` opens `WelcomePane` when
  the workspace has restored and the welcome preference is enabled.
- Observed: `just local-install` built and installed the app successfully,
  including replacement of an earlier install. `file` reports a Mach-O arm64
  executable and `codesign --verify --deep --strict` accepts the installed
  bundle. Launched `/Applications/Hickory Docs.app` through the native app
  interface: Welcome was selected, its Start actions were visible, and the
  four restored document tabs remained available. The two existing Vitest
  suites `toolTabs.test.ts` and `workspaceState.test.ts` passed (38 tests);
  the release build also passed the frontend TypeScript check.
- Test coverage: manual installation and native UI inspection. A unit test
  would not establish macOS app discovery or WebKit rendering. Interrupted
  replacement and unavailable build prerequisites require implementation
  review; this is not a crash-safe installation transaction.
