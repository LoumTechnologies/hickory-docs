<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/authoring/hick-open-hands-a-folder-to-the-app.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `18e7e24295498b7c3d95fbf491bb1e4fdc8f20331efce5bdbb1634cdefa6a1ac`.

<book:entry path="docs/guarantees/authoring/hick-open-hands-a-folder-to-the-app.md">
# hick Opens The App While Explicit Verbs Keep Their CLI Behavior

Given an installed desktop app, when `hick` runs without arguments, then it
launches a blank editor window and returns. `hick <path>` and `hick open <path>`
open the named folder or document with an absolute path; a missing path is refused.
`hick open` defaults to the current folder. Existing subcommands and aliases,
`--help`, and `--version` keep their CLI behavior. A command name wins over a
bare folder name; `./test` or `hick open test` disambiguates that folder.

A misspelled command without a path shape or an existing file still gets clap's
command error and suggestion. Paths can contain spaces.

App discovery prefers `HICKORY_DESKTOP`, then the app beside or enclosing the
CLI, then platform locations, then PATH. An explicit missing app is refused
by name. CLI-only installations explain how to get the desktop app while
preserving headless commands. Launches clear development-session overrides.

---

Last LLM verification:

- Date: 2026-10-03
- Reviewer: Codex
- Result: partially verified
- Evidence: `cli_launch.rs::normalize` and `open`, `main.rs::cmd_open`, and
  `open_app.rs::{find, open, open_blank, not_installed}`.
- Test coverage: binary normalization tests and `tests/open_app.rs` launch
  a stub process, verify blank/document arguments and CLI help/errors;
  `open_app` unit tests check app discovery.
- Caveat: the real packaged macOS app is checked separately from these stub
  tests. Windows and AppImage launch behavior require their native runners.
</book:entry>

</book:doc>
