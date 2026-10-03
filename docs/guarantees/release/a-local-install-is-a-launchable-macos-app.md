# A Local Install Is a Launchable macOS App

Given a checkout on an Apple Silicon Mac with the desktop build prerequisites
and write access to `/Applications`, when `just local-install` succeeds, then
`/Applications/Hickory Docs.app` contains a native release build with the UI
embedded and is registered with macOS as **Hickory Docs**. It can be launched
from Applications without a running development server or this checkout.

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
Hickory Docs, with Files and Agent available on demand. Startup ignores the
previous pane arrangement and Welcome preference. See
`../authoring/the-app-starts-as-a-lightweight-editor.md`.

---

Last LLM verification:

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
