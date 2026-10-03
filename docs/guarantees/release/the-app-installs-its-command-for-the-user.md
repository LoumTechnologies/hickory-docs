# The App Installs Its Command For The User

Given a packaged desktop app, then its bundle includes the real `hick` CLI.
When the user installs the command through Settings → Command line, then it
becomes available on the user's PATH in a new terminal, without requiring a
system PATH change. The command palette provides a route to those settings.
An existing conflicting command is identified and is never overwritten.

Installation is idempotent. Removal deletes only this app's launcher and its
unchanged, marked PATH blocks; unrelated shell settings remain intact. Linux
packages install the CLI in their normal executable directory. An AppImage's
launcher invokes the saved AppImage, never a transient mount. Windows MSI
installation offers a checked-by-default user PATH checkbox and removes its
PATH entry on uninstall. Settings can also add/remove the Windows user entry.

The shell installer adds its directory to a recognized shell's startup profile
when it is absent from PATH; `HICKORY_MODIFY_PATH=0` opts out. Existing marked
blocks are preserved.

---

Last LLM verification:

- Date: 2026-10-03
- Reviewer: Codex
- Result: partially verified
- Evidence: desktop `command_path` modules and same-origin routes; Settings
  `CommandPathSettings`; `stage-desktop-cli`, `dist-desktop.sh` and the MSI
  `windows/command-path.wxs` fragment; `main.rs` AppImage CLI delegation;
  `scripts/install.sh::setup_path`.
- Test coverage: isolated Unix install/reinstall/removal, conflict and edited
  profile preservation, AppImage launcher quoting; Settings installation,
  removal, conflict and failure UI; desktop route origin rejection. A locally
  signed macOS debug bundle passed `codesign --verify --deep --strict`, bundled
  CLI help/version, and install → execute CLI → reinstall → remove through its
  live HTTP endpoint with an isolated temporary home. Release CI checks bundled
  CLI artifacts.
- Caveats: Windows registry writes and MSI checkbox/uninstall need a native
  Windows runner. AppImage mounting needs Linux. These are not verified by
  macOS-only tests. A moved application needs command reinstallation.
