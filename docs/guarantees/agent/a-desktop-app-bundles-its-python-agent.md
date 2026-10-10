# A desktop app bundles its own Python agent

Given a macOS desktop build produced by `just local-install`, `just
build-desktop-app`, or `scripts/dist-desktop.sh`, when the app is installed
and opened, then the experimental Python agent is available without naming a
path: the app carries a self-contained Python runtime at
`Contents/Resources/python-agent`, discovered from the running executable's own
location. **Settings → Agents** says it is using that runtime, the Agent pane
offers **Hickory Python (experimental)** as available, and no value is written
into `python.json`.

The bundled runtime is built by `scripts/build-python-agent-runtime.sh` from
the locked environment under `apps/agent`: the relocatable CPython that `uv`
installed, the environment's installed packages, and the `monty` interpreter,
folded into one directory whose `bin/python` resolves inside itself.
`bundle.macOS.files` copies it through Tauri's `copy_dir`, which preserves the
relative symlink `bin/python -> python3.14`; a copy that lost the symlink or
the `pyvenv.cfg` marker would be refused by
`hickory_executor_sandbox::agent_process`.

A runtime named in Settings always wins over the bundled one, and clearing the
field returns to the bundled runtime. When a desktop app opens a workspace
served by an engine another program started — specifically `hick up`, which has
no bundle to point at — the app sends its bundled runtime path in the attach
request, so the engine uses it too.

---

Last LLM verification:

- Date: 2026-10-10
- Reviewer: Codex
- Result: verified for the macOS `.app` on this Apple Silicon Mac; the `.dmg`
  path shares the same build script and was not re-bundled into a disk image
  here. Notarized-release nested-code signing was implemented but not exercised.
- Originating requirement: the user asked that `just local-install` and the
  dmg installer include the Python agent so it can be tried without setting a
  path in Settings.
- Implementation: `crates/hickory-cli/src/serve/python_agent.rs`
  (`effective_runtime`, `bundled_runtime`, `bundled_runtime_for`) resolves the
  runtime; `crates/hickory-cli/src/serve/mod.rs` carries
  `LocalState::python_runtime_path`; `crates/hickory-cli/src/engine/mod.rs`
  adds `Attach::python_runtime_path` and `engine/daemon.rs` applies it;
  `apps/desktop/src-tauri/src/server.rs` sends it; `scripts/dist-desktop.sh`
  and `justfile` stage and bundle the runtime.
- Witnesses:
  - `serve::python_agent::tests::a_runtime_is_found_inside_an_app_bundle_and_beside_a_bare_executable`
    and `nothing_is_found_when_no_runtime_was_shipped` challenge the path
    logic against a fake bundle.
  - `serve_python_agent::runtime_setup_refuses_invalid_installations_and_reserved_acp_ids`
    asserts the Settings route reports `available: false` and `bundled: null`
    when no runtime is configured and none is shipped.
  - `AgentSettings.test.tsx` asserts the bundled path is shown as a hint while
    the field stays empty and overridable.
- Executed checks:
  - `./scripts/build-python-agent-runtime.sh` assembled the runtime and its
    `bin/python -I -B -c 'import hickory_agent_worker'` import check passed; the
    assembled tree was ~150 MB.
  - `just build-desktop-app` built the debug `.app` with the runtime at
    `Contents/Resources/python-agent`; the relative `bin/python` symlink
    survived, `codesign --verify --deep --strict` passed, and the worker
    imported from the bundle.
  - The existing real confined-worker test
    (`serve_python_agent::python_turn_requires_review_streams_receipts_and_rehydrates_its_branch`)
    passed with `HICKORY_AGENT_TEST_PYTHON` pointed at a relocated runtime and
    a copy whose path contained a space.
  - Starting the bundled `hick up` engine and reading
    `/api/agents/python` returned
    `{"available":true,"bundled":"…/Hickory Docs.app/Contents/Resources/python-agent/bin/python","python":""}`,
    and the catalogue listed the Python agent with `available: true`.
- Assumptions/exclusions: macOS only — `bundle.macOS.files` and
  `bundled_runtime`'s `Contents/Resources` candidate are macOS-shaped, and the
  Linux/Windows desktop bundles do not ship the runtime yet. A cross-compiled
  Intel macOS bundle built on an arm64 machine is also skipped: `uv` installs
  the host's interpreter, so bundling it would put an arm64 Python in an
  x86_64 app. The `Attach::python_runtime_path` hand-off to a CLI-started
  engine was reviewed but not exercised with two processes. The runtime is a
  full CPython plus the environment (~150 MB); no trimming beyond the
  interpreter's own layout is done. The worker's confinement is unchanged and
  remains `python-execution-has-two-boundaries.md`.
- Challenges: a build that lost the symlink or `pyvenv.cfg`, a `uv` version
  that stopped writing an absolute interpreter `home`, a Python minor bump
  that changed `lib/pythonX.Y`, or a `.dmg` produced without the staged
  runtime would falsify the relevant claim. A signed release whose nested
  Mach-O files are not pre-signed would fail notarization; the build script
  signs them when `APPLE_SIGNING_IDENTITY` names an identity.
- Dependencies: `apps/agent/uv.lock` and `apps/agent/pyproject.toml` fix the
  environment; `scripts/build-python-agent-runtime.sh` owns the assembly;
  `uv` must be installed to build. Changing how Tauri copies custom files
  (currently `copy_dir`, which preserves symlinks) would break the layout.
- Inspected SHA-256 dependencies (2026-10-10):
  - `crates/hickory-cli/src/serve/python_agent.rs`: `a3b9f0b8eefb31142716ce1bcf8cda033bfc89f8a524d05ada21114b5a84afd8`
  - `crates/hickory-cli/src/serve/mod.rs`: `f0151740d1bd08568cef224a3e494e25e82687751261b624528f0921c787f802`
  - `crates/hickory-cli/src/engine/mod.rs`: `84e10349a21e7df4fc4fdb2736cdc8aa88ad9b8a68207ef3e2783dec5978a02b`
  - `crates/hickory-cli/src/engine/daemon.rs`: `2d4a151c6c6bd8611713294b934c00a2cdd8aab602c2da01028a9466448a0343`
  - `apps/desktop/src-tauri/src/server.rs`: `b035dc73be23a0e720ec70a2e912879447dbdf4e4a85f0657c358461294600cc`
  - `scripts/build-python-agent-runtime.sh`: `795f1aead5d23cb23c0ed17830ef37e5431984ef84b991659596fdc87a5fd2a5`
  - `scripts/dist-desktop.sh`: `7c9b5ca13dc63a3ff56125e73b8933b516f1ad28dd3c8c73c14e17e106102367`
  - `justfile`: `76fd4618c5de989650b72fd63eb56c038be94c2b3e146fd48c41f51fdd0e8c08`
  - `.github/workflows/release-build.yml`: `68db790a5eb0c6c6f7a579e299822ce0d9082ce0e8b477f606ca03b35adfc570`
  - `apps/web/src/views/AgentSettings.tsx`: `ad7cade7d4db241cba6324882b58a44509d811738aced39fe53c60107f072429`
  - `apps/web/src/views/AgentSettings.test.tsx`: `e00c0fd81c163c9ee62b4838443056eaf705e1e05f6827d6651c7acdc01e2846`
  - `crates/hickory-cli/tests/serve_python_agent.rs`: `fafc614f7119e96b4234be2979eb5867c2ec2c8e0004618ac3a5e877164ab2a3`
- Human judgment: not recorded; automated reviewers are fixtures, not evidence
  of a person's semantic approval.
