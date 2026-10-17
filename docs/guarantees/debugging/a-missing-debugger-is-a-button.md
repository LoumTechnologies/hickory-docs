# Debug installs a missing supported adapter and continues

Given a developer clicking the gutter's Debug action, when this machine is
missing an adapter available from Hickory's catalogue, then the app installs it
and starts the requested program without another click. Setup reports its state
in a floating message beside the right-gutter controls. A failed installation shows its diagnostic and a
Retry installation action. Dismissing setup prevents a late download from
starting a session. Repeated missing-adapter failures do not trigger an automatic
installation loop.

Only a typed, catalogue-supported missing-adapter response triggers installation.
For tools supplied by another ecosystem, the engine's installation guidance
remains visible. Installation uses the same confinement, prefix and catalogue
as `hick dap install`. Concurrent requests for one adapter share the installation.

This supersedes the separate Install button decision on 2026-10-16: requesting
Debug authorizes the prerequisite installation. The user should choose the
program, not manage its adapter before each first run.

On macOS, the redirected installer home applies to every command in the
install, including creating the Python environment and installing debugpy.
Retrying reuses the Python environment left by a previous failed install.
If installation fails, the app receives the installer's stdout and stderr
along with the exit status and a suggestion to retry.

The shared install prefix is shell-quoted. Spaces, quotes and shell punctuation
in a project's path remain path bytes, including when an installer appends a
subdirectory such as `python/bin/python`. This applies equally to language-server
and archive installers.

The offer is cleared the moment a session starts. It belongs to the failure it
came from, and an install that worked must not leave "No Python debugger on
this machine" standing beside "finished — exit code 0".

Deciding *which* failure this is must not be done by reading English.
`hick_dap::MissingAdapter` is a type for exactly that reason: the server
downcasts to it, asks it whether the language is installable, and the client
gets a field. A message somebody rewords cannot silently take the button away.

---

Last LLM verification:
- Date: 2026-10-01
- Reviewer: Codex
- Result: verified
- Evidence:
  - On macOS, reproduced the debugpy install failure: the second command
    tried to write `~/.cache/uv`, outside the writable prefix. Exporting HOME
    for the whole shell fixed it. A confined install of debugpy 1.8.21 and
    a retry succeeded; all ten real Python DAP session tests passed.
  - The type: `crates/hick-dap/src/program.rs::MissingAdapter`, returned by
    `adapter_for`, carrying `language`, the prose `how`, and `installable`
    (from `discovery::suggests_hick_install`, the same function that writes
    the sentence — so the two cannot disagree).
  - The wire: `crates/hickory-cli/src/serve/debug_bridge.rs` — `Response::Failed`
    gained `offer_install`, set by downcasting the error and filtering on
    `installable`.
  - The install: `crates/hickory-cli/src/serve/install.rs` — `POST /api/install`
    calls `crate::dap_install::install`, the same entry point `hick dap
    install` uses, on a blocking task so a download does not hold a runtime
    worker.
  - The control: `apps/web/src/debug/DebugStrip.tsx::MissingTool`, wired in
    `apps/web/src/views/workspaceTabs.tsx` to install and then re-start the
    same program.
  - Run end to end on this machine against `warehouse/50-review-surface.md`:
    the strip showed "No Python debugger on this machine. [Install it]",
    the click produced `.hick-cache/adapters/python/` and the session then ran
    to `finished — exit code 0`.
- Test coverage: `apps/web/src/debug/DebugStrip.test.tsx` — the offer replaces
  the sentence and reports the pick; a language with no offer still shows the
  prose in full (the `delve` case). `apps/web/src/debug/useDebugger.test.ts` —
  the offer is carried out of the failure and **dropped when a session
  starts**, which is the bug the `clearFailure` pairing exists to prevent.
  `crates/hick-dap/src/program.rs::tests::a_missing_adapter_is_a_type_the_app_can_act_on`
  — the failure downcasts, names the language, and its Display is unchanged.
  `a_compound_installer_keeps_its_home_for_every_command` checks both commands
  inherit the confined home, including a path with spaces and an apostrophe;
  on macOS it also runs through Seatbelt.
  `a_failed_installer_returns_its_diagnostics` checks that the install error
  includes stdout, stderr and the next step without needing a network.
- Caveat requiring LLM review: the install is blocking with no progress. A
  pinned-hash download of a debug adapter takes seconds, and a progress
  protocol for that wait would cost more than it returns — but a very slow
  connection gets "Installing…" and nothing else. There is also no offer for
  a missing **language server**, which is the same gesture and is not wired.

Verification update (2026-10-13): an installed-app drive reproduced `uv` rejecting
the second word of a project folder containing spaces. `tool_install.rs` now
substitutes a shell-quoted prefix. The offline confined regression
`an_install_prefix_is_one_shell_word_even_with_spaces_and_punctuation` writes and
reads a receipt through a prefix containing spaces, an apostrophe, `$()` and a
semicolon. All seven installer tests and the confined-home test passed.


Verification update (2026-10-16): `DebugStrip.tsx::MissingTool` automatically
installs and resumes, shares in-flight installations, and guards continuation
against dismissal and unmount. `DebugStrip.test.tsx` covers successful setup,
installer failure and explicit retry, dismissal during download, and repeated
missing-adapter responses. Installer and protocol behavior are unchanged; this
update does not claim a new live provider download was exercised.
