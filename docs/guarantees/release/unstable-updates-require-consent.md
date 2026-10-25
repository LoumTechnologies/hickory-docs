# Unstable Updates Require Consent

Given a packaged release build for Apple Silicon macOS whose SemVer prerelease
channel is `unstable`, when its editor window has opened, then the app checks
the configured HTTPS GitHub Unstable manifest and checks again every 30 minutes.
Stable builds, development builds, and other platforms do not start this loop.

Given a newer Unstable version, downloading and installing requires affirmative
consent in the native update dialog. The Tauri updater verifies the archive
signature using the public key embedded at build time. The update installs on
disk without forcing a restart, so save/discard and workspace detach handling
remain in control of shutdown. Network errors do not prevent editor startup.

---

Last LLM verification:

- Date: 2026-10-25
- Reviewer: Codex
- Result: partially verified.
- Originating requirement: user request to periodically detect Unstable uploads,
  prompt the user, and automatically download/install accepted updates.
- Evidence: `apps/desktop/src-tauri/src/updater.rs` (`start`, `is_unstable`,
  `macos::check`), desktop setup in `src/lib.rs`, and
  `scripts/build-unstable-macos.sh` supplying the updater key and endpoint.
- Assumptions: the initial DMG is manually installed; its public key matches
  the signing key; the builder is trusted; the repository is publicly readable;
  the app bundle is writable or macOS authorizes its replacement. Updating
  means installing the bundle now and using it after the next normal launch.
- Exclusions: stable/non-macOS update support and forced relaunch were not
  requested. No frontend capability grants updater access to hosted content.
- Witnesses and challenges: the channel unit test rejects stable, beta,
  lookalike channel names, and build metadata containing `unstable`.
  Source review places download/install after the affirmative dialog result,
  uses a cross-process file lock during installation, and never calls restart.
  A mismatched key, offline startup, failed permissions, or simultaneous folder
  processes on a real Mac remain operational challenges.
- Test coverage: two policy tests reject duplicates/downgrades/other channels
  and check timestamp ordering independently of hash ordering; native dialogs, signature
  rejection, and bundle replacement need a real macOS run and are unverified.
- Dependencies: source hashes in `unstable-publication-review.json` pin the
  inspected implementation. The plugin dependency is locked to updater 2.12.0
  in the desktop Cargo.lock; changes stale this review.
- Executed checks: Linux library compilation passed. `just test-unstable-release` passed eight publisher/signature-staging tests
  and two policy tests. The updater API body also compiled on Linux with its
  macOS platform guard removed in a temporary harness. This checks shared API
  types, not macOS installer internals. Results are recorded in the dependency audit. No macOS build
  or real install executed; no broad regression checks run or monitored.
- Human judgment: not recorded.
