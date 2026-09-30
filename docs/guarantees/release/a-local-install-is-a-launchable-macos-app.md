# A Local Install Is a Launchable macOS App

Given a checkout on an Apple Silicon Mac with the desktop build prerequisites
and write access to `/Applications`, when `just local-install` succeeds, then
`/Applications/Hickory Docs.app` contains a native release build with the UI
embedded and is registered with macOS as **Hickory Docs**. It can be launched
from Applications without a running development server or this checkout.

A failed build or staging copy leaves an existing installed app untouched.
Re-running the recipe replaces the installed bundle. User documents and
settings are not part of that replacement.

The welcome pane becomes active after the workspace restores, with the
restored tabs still available. Unchecking “Show this page when a folder opens”
disables this startup behavior.

---

Last LLM verification:

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
