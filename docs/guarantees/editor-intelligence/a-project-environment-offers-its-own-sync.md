# A project environment offers its own sync

Given a host project identified as uv, npm or pnpm by its manifest,
configuration or lockfile, when Hickory checks its environment, then it
reports a typed finding and offers only a repair the selected provider can
plan. An ambiguous manager offers an explicit choice, saved in the user's
workspace state, before it offers installation.

For uv, missing environments, native-check-confirmed stale environments,
missing/stale locks, missing managers and inconclusive checks are distinct.
An existing virtualenv alone does not mean Ready. Unsupported structured
checks, timeout and offline uncertainty remain inconclusive. The selected
profile is the package manager's defaults, including its inherited options.

For npm and pnpm, the initial provider identifies missing installation only.
An existing `node_modules` is reported as unverified, never Ready. Supported
workspace declarations determine shared ownership; independent nested
projects keep their own findings. Bare manifests do not silently select a
manager. Unsupported managers have no install action.

Opening a project, polling or pressing Recheck performs no installation.
Automatic uv probes disable networking, Python downloads and builds, use a
temporary cache, and use check/locked options. They do not update manifests,
lockfiles or project environments. Explicit repairs may download packages,
build them and execute their lifecycle code.

Given an offered action, when the person presses its button, then the command,
directory and effects are visible and the package manager runs in a retained
terminal session. The engine rediscovers and rechecks the project, rejects a
stale finding or an unknown action, and serializes repairs to one project
directory. The client cannot supply shell text or an undiscovered directory.
Completion invalidates cached readiness; success alone does not mark Ready.

After uv prepares a host environment, plain-file Python test and debug
launches select its interpreter. Supported Python language servers receive
the interpreter through configuration; existing servers are refreshed after
a repair or a readiness transition. This does not install a debugger or a
language server. Document cells and staged document debug runs retain their
executor/scratch boundaries: a host sync is never reported as proof that a
cell target is ready.

Findings appear in Project environments within Problems, with a workspace
notice for known problems and an Environments button in the status bar.
Unknown results stay in the expandable panel. Dismissing a notice applies to
its evidence revision; repeated diagnostic timings do not reopen it.

---

Last LLM verification:

- Date: 2026-09-30
- Reviewer: Codex
- Result: partially verified
- Evidence: `hick-project-env::Registry`, `providers::uv::Uv`,
  `providers::node::Node`, `host::Host`; `serve::environments::{inspect,act,choose}`;
  `serve::test_run::test_command`, `debug_sessions::Registry::start_plain`;
  `hick-lsp::{project_environment,child_lsp,dispatcher}`;
  `apps/web/src/environments/{useEnvironments,EnvironmentPanel}.tsx` (hook is `.ts`).
- Test coverage: `crates/hick-project-env/tests/providers.rs` covers manager
  evidence, workspace boundaries, native result states, unchanged files,
  timeouts and stable revisions. `crates/hickory-cli/tests/serve_environments.rs`
  drives real HTTP and a real uv terminal against a local wheel: absent
  environment, unchanged lockfile, synchronization, retained output, Ready,
  dependency import through the test interpreter, stale-action rejection and
  a manifest/lock mismatch. `EnvironmentPanel.test.tsx` covers visible commands,
  actions, dismissal, a busy terminal and cleared notices. Existing Problems
  and StatusBar tests remain in the relevant regression suite.
- Caveats: the live uv run was on macOS with uv 0.12.21. Native JSON output is
  currently a preview schema; unrecognized shapes degrade. An offline probe
  can be inconclusive even for a stale lock when metadata cannot be resolved;
  this was observed for a changed project with a local wheel. Native manager
  checks establish package-manager readiness, not integrity of every installed
  byte. Real npm/pnpm repairs, Windows terminal behavior, and a live Python DAP
  session for this new interpreter selection were not exercised here. Python
  LSP settings/configuration replies are unit-tested; refresh does not replace
  an already-running language-server binary newly installed by sync. Alternate
  dependency profiles have no selection UI yet. Independently distributed
  third-party plugins and non-host environment probes are not implemented.
