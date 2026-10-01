# The Desktop Finds Tools From The Shell PATH

Given a macOS desktop launch from Finder or the Dock, when the person's login
and interactive shell configuration produces an absolute executable search
path, then Hickory imports that PATH before starting its engine. GitHub's `gh`
commands and other child processes use that same path. Shell order is preserved;
inherited-only directories follow it. No package-manager location is hardcoded.
Profile banners do not become paths, and no credentials are imported.

Given a default zsh terminal pane on macOS, when it starts, then it reads login
profiles as well as interactive configuration. Hickory's profile forwarding
keeps its directory and command hooks active through `.zprofile`, `.zshrc`, and
`.zlogin`. An explicitly requested command retains its requested arguments.

Given failed, malformed, excessively noisy, or hanging PATH discovery, when
the desktop starts, then it retains its inherited PATH. Discovery has a
five-second timeout and a 64 KiB output limit. A failure to launch `gh` states
the launch error and useful next steps rather than claiming it is uninstalled.

---

Last LLM verification:

- Date: 2026-10-01
- Reviewer: Codex
- Result: verified
- Evidence: `apps/desktop/src-tauri/src/main.rs` initializes PATH before Tauri;
  `launch_path.rs` runs the shell, frames its PATH, and bounds discovery.
  Engine children inherit the environment through `engine/client.rs`.
  `hick-term/src/session.rs` starts default macOS zsh with `-l`;
  `shell_integration.rs` retains its generated startup directory until the
  final startup file. `serve/github.rs::command` executes directly and
  preserves launch errors.
- Test coverage: `just test-binary-discovery` exercises real zsh startup with
  a minimal GUI PATH and a temporary executable, profile banners, non-UTF-8
  paths, order, invalid frames, failures, and timeouts. Terminal checks cover
  login profile forwarding, directory tracking, and typed command reporting.
- Caveat: arbitrary user startup scripts may themselves fail or wait for a
  terminal; PATH recovery then degrades to the inherited environment. A
  running shared engine retains its startup environment until it restarts.
