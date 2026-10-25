<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/authoring/the-daemon-exits-with-its-last-client.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `2268eea668520965a885f40a266fb014a296af485ccacafe4e0cb1cc305f296a`.

<book:entry path="docs/guarantees/authoring/the-daemon-exits-with-its-last-client.md">
# The daemon exits with its last client

Given a local Hickory daemon, when its last desktop window or CLI client
disconnects, then the daemon shuts down. A surviving client keeps the shared
daemon alive. Open terminals, workspace runs, and private investigations do not
independently extend its lifetime.

Explicit disconnects are detected on the two-second lifecycle sweep. Crashed
clients expire after ten seconds without a heartbeat. One-shot CLI commands
renew their own leases while waiting for execution; their workers alone cannot
keep an abandoned daemon alive. Before any client has connected, startup has a
ten-second grace period. Shutdown allows up to two seconds for HTTP draining,
then closes terminal sessions, resets retained debuggers, releases generated-file
protection and removes discovery. The kernel-owned lock file is preserved.

---

Last LLM verification:

- Date: 2026-10-23
- Reviewer: Codex
- Result: partially verified; six focused real-process lifecycle tests passed.
- Originating requirement: the user requested that the daemon exit when the
  last Hickory Docs window or CLI exits, replacing the installer process check.
- Interpretation: process crashes require bounded lease expiry; background work
  is owned by clients rather than an independent reason to retain the daemon.
- Evidence: `engine/daemon.rs::run`, `attach`, `command_lease`, `command_detach`,
  and `command`; `engine/client.rs::guard`, `ClientGuard::drop`, `forward_cli`.
  Paths are beneath `crates/hickory-cli/src`.
- Test coverage: `crates/hickory-cli/tests/engine_lifecycle.rs` tests
  `daemon_exits_after_last_detach_even_with_an_open_terminal`,
  `daemon_expires_crashed_clients_but_keeps_the_remaining_window`, and
  `daemon_exits_after_a_one_shot_cli_command`, plus
  `daemon_expires_an_abandoned_cli_even_while_its_worker_runs`,
  `daemon_cancels_background_preparation_after_last_client_exits`, and
  `daemon_reopen_waits_for_the_retiring_owner_and_starts_fresh`.
- Assumptions and challenges: async detach during process teardown may not
  complete; lease expiry covers that case. Shutdown admission is serialized
  with attaches and CLI leases so a newly accepted owner cannot be overlooked.
  Foreground preparation renews its lease while attach waits; watcher startup
  completes outside the admission lock. Discovery retries kernel lock ownership
  while an earlier daemon drains, and window/CLI admission retries retirement
  refusals instead of attaching to a stopping owner.
  Engine protocol 2 requires the previous daemon to be stopped once before
  upgraded clients can connect. This change does not remove old app bundles or
  change macOS Dock shortcuts. Native close gating remains in place.
- Coverage gaps: no automated native macOS window interaction or exhaustive
  subprocess-tree cleanup assertion; lease timing assumes the daemon scheduler
  can make progress. Retained debugger cleanup can extend shutdown time.
- Dependencies (SHA-256 of inspected content):
  - `engine/client.rs`: `7f1a83780f4457caec3bb7bef39b0d8c73ca4aaa43169b59e8845dd7c60442b6`
  - `engine/daemon.rs`: `3a4edb9385f3f08962764250b5162177c5d0efa4114462815b935927ac73ace8`
  - `engine/mod.rs`: `3c2384eecd7c161edd973e9d2e9fd75ef5ebc153119519ada4c19a1d4a12ab79`
  - `engine/watch.rs`: `27b530bbdd751a6631abab4205fb0d044a95ac426ff4102a8eeafc0ac40e487d`
  - `tests/engine_lifecycle.rs`: `f9cd8f838b8de364da3c0b8681b98134165a3a598d65c345c34d71ed8981594f`
  Implementation paths above are beneath `crates/hickory-cli/src`; the test is
  beneath `crates/hickory-cli`. Changes invalidate this evidence.
- Executed checks: release build of `hick` and `engine_client` passed; six tests
  selected by `cargo test -p hickory-cli --test engine_lifecycle daemon_` passed
  (49.23s). The actual commands used `--release --target aarch64-apple-darwin`,
  `CARGO_TARGET_DIR=apps/desktop/src-tauri/target`, and Cargo configuration
  overrides `profile.release.package.hickory-cli.opt-level=0` and
  `profile.release.package.hickory-cli.codegen-units=16`. The initial debug build
  stalled in cached dependencies; the initial fully optimized build was stopped
  in favor of this focused build. Neither was counted as a passing check.
  Existing either-order CLI/window attachment, background editor bootstrap and
  authentication/stale-discovery tests also passed. The existing
  `prediction_engine::every_engine_client_inherits_private_policy_and_mcp_and_command_paths_cannot_bypass`
  safety check passed (3.01s), using a synthetic provider and actual daemon.
  Broad regressions and native UI checks were not run.
- Human judgment: not recorded.
</book:entry>

</book:doc>
