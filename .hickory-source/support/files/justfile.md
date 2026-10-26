<maint:doc xmlns:maint="http://www.hickorydocs.com/1.0">
# Maintain justfile

For: maintainers changing the literate workflow and reviewers checking migration evidence.

Edit this entry, regenerate, and commit its output with the Markdown. Historical evidence retains its original scope.

<maint:entry path="justfile">
# Hickory Docs task runner — the only entry point for project tasks.

# Run local checks, with compact progress and per-commit results for clean trees.
checks:
    @scripts/checks.sh

# Exercise local runner attribution, isolation, locking and hook detachment.
test-checks:
    @scripts/checks.sh --test

# Show HEAD’s recorded result and the latest local run without running checks.
checks-status:
    @scripts/checks.sh --status

# Build the whole Rust workspace.
build:
    cargo build --workspace

# Type-check quickly.
check:
    cargo check --workspace

# Run all tests.
test:
    # Build first (as CI does): the server's LSP tests spawn the `hick-lsp`
    # binary, which `cargo test` alone does not produce.
    cargo build --workspace
    cargo build -p hickory-cli --example acp_fixture
    cargo test --workspace -- --test-threads=1
    just test-python-agent

# Confined tool installers and the diagnostics returned to the app.
test-tool-install:
    cargo test -p hickory-executor-sandbox --test installer_home
    cargo test -p hickory-cli --lib tool_install::tests
    cargo test -p hick-dap --test live_session -- --test-threads=1
    cargo build -p hickory-cli

# Regenerate everything derived from another file in this repo.
#
# Includes the language table, editor parser WASM and shared context wire types.
codegen:
    scripts/check-codegen.sh --write

# Fail if anything generated is out of date with its source. Runs in CI and in
# the local checks, so each run records generated-file drift.
check-codegen:
    scripts/check-codegen.sh

# No source file over a thousand lines, and the ones already over only
# shrink — see scripts/check-file-length.sh. `--write` lowers a baseline
# after a split.
check-file-length *ARGS:
    scripts/check-file-length.sh {{ARGS}}

# Lint (warnings are errors, matching CI).
clippy:
    cargo clippy --workspace -- -D warnings

fmt:
    cargo fmt --all

# Everything CI runs, in one command.
#
# NOT `verify` — that name already means `hick test <doc>`, which verifies a
# DOCUMENT. This verifies the repository. The point of one entry is that there
# is never a gap between what a hook runs and what CI runs, since that gap is
# where "passed locally, failed in CI" comes from; keep this and
# `.github/workflows/ci.yml` saying the same thing.
#
# `cargo build` before `cargo test` is load-bearing and not belt-and-braces:
# the LSP tests spawn the `hick-lsp` binary, which `cargo test` alone does not
# produce, and without it they fail claiming no language server is installed.
ci:
    python3 scripts/generate-api.py --check
    just test-review-complexity
    if [ "$(uname -s)" = Darwin ] && [ "$(sw_vers -productVersion | cut -d. -f1)" -ge 26 ]; then just build-fskit; fi
    just check-codegen
    just check-file-length
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo build --workspace
    cargo build -p hickory-cli --example acp_fixture
    cargo test --workspace -- --test-threads=1
    cd apps/web && npm run typecheck && npm test -- --maxWorkers=2
    just install-test-browsers
    just test-browser-embedding
    just test-python-agent

# Hot reload for the homepage, separate from the desktop app's entry.
dev-site:
    cd apps/web && npm run dev:site

# Build the static marketing site into apps/web/dist-site — no server, no accounts,
# no billing. Deploy the directory anywhere that serves files.
#   just site                      # analytics disabled
#   POSTHOG_KEY=phc_… just site    # with browser-side capture
site:
    #!/usr/bin/env bash
    set -euo pipefail
    key="${POSTHOG_KEY:-}"
    # Fail before building, not in the visitor's browser: a personal API key
    # (phx_…) inlined into a public bundle would hand out project admin.
    if [ -n "$key" ] && [ "${key#phc_}" = "$key" ]; then
      echo "POSTHOG_KEY must be a PostHog PROJECT write key (phc_…), not '${key:0:4}…'." >&2
      echo "A personal API key (phx_…) can create and destroy projects and must" >&2
      echo "never reach a browser bundle. See docs/operators/analytics.md." >&2
      exit 1
    fi
    # The install command the site advertises is
    # `curl -fsSL https://hickorydocs.com/install.sh | sh`, so the site has to
    # serve that script. Copied at build time rather than committed twice:
    # scripts/install.sh stays the single source.
    mkdir -p apps/web/public
    cp scripts/install.sh apps/web/public/install.sh
    cd apps/web
    HICKORY_SITE_ONLY=1 VITE_POSTHOG_KEY="$key" npm run build:site
    echo
    echo "Static site built: apps/web/dist-site"
    echo "It needs no backend. The landing page and its demos run in the browser."
    echo "Serves /install.sh — note that the installer pulls from GitHub releases,"
    echo "which strangers can only reach once the repository is public."

# Hot-reloads the UI; the engine runs in the app's own process; closing the
# window stops everything.
# The desktop app, in dev mode, on a seeded scratch project.
dev:
    ./scripts/dev.sh

# Drive the Files editor against the live app started by `just dev`.
test-e2e:
    HICKORY_E2E_URL="http://127.0.0.1:$((41000 + $(pwd | cksum | cut -d' ' -f1) % 8000))" npm --prefix apps/web run test:e2e

# Stop anything a killed `just dev` left behind.
dev-stop:
    ./scripts/dev.sh stop

# Never another worktree's, and never a cache anyone would re-download.
# Stop, then delete THIS worktree's dev scratch (.dev/).
dev-clean:
    ./scripts/dev.sh clean

# There are no accounts to seed — there is no server to sign in to.
# Idempotent seed data: a two-stage chain of real documents for the app to open.
dev-seed:
    ./scripts/dev-seed.sh

# Build one downloadable artifact (hick + hick-lsp + LICENSE + examples) into
# dist/.
# The same script the release workflows call, so a maintainer can reproduce
# what CI ships. Targets:
#   x86_64-unknown-linux-gnu   aarch64-unknown-linux-gnu
#   aarch64-apple-darwin       x86_64-apple-darwin
#   x86_64-pc-windows-msvc
dist TARGET VERSION="":
    ./scripts/dist.sh {{TARGET}} {{VERSION}}

# Build the downloadable DESKTOP app for this platform into dist/desktop/.
# A separate download from the CLI, because a GUI application is a .dmg, an
# .msi, or a .deb/.AppImage — not a tarball of binaries. Needs cargo-tauri:
#   cargo install tauri-cli --version '^2' --locked
dist-desktop VERSION="" TARGET="":
    ./scripts/dist-desktop.sh {{VERSION}} {{TARGET}}

# Publish HEAD to Unstable through an Apple Silicon SSH builder; no Actions.
# Set HICKORY_MAC_BUILDER; see apps/desktop/RELEASING.md for one-time setup.
release-unstable:
    python3 scripts/release-unstable.py

# Focused publication ordering, signature staging, and update-channel safety tests.
test-unstable-release:
    python3 -m unittest discover -s scripts/tests -p test_release_unstable.py -v
    cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --lib updater::tests --locked
    bash -n scripts/build-unstable-macos.sh scripts/dist-desktop.sh

# Build and install the native macOS app in Applications (no disk image needed).
local-install:
    #!/usr/bin/env bash
    set -Eeuo pipefail
    phase="checking prerequisites"
    trap 'status=$?; echo "local-install failed while $phase (exit $status). See ${log:-the output above}." >&2' ERR
    if [ "$(uname -s)" != Darwin ] || [ "$(uname -m)" != arm64 ]; then
      echo "local-install requires an Apple Silicon Mac." >&2
      exit 1
    fi
    if ! command -v cargo-tauri >/dev/null 2>&1; then
      echo "Install the desktop build tool first: cargo install tauri-cli --version '^2' --locked" >&2
      exit 1
    fi
    if ! command -v uv >/dev/null 2>&1; then
      echo "Install uv first (the bundled Python agent is built with it): https://docs.astral.sh/uv/getting-started/installation/" >&2
      exit 1
    fi
    if [ ! -w /Applications ]; then
      echo "local-install needs write access to /Applications." >&2
      exit 1
    fi
    mkdir -p .dev
    log="$PWD/.dev/local-install.log"
    exec > >(tee "$log") 2>&1
    echo "Install log: $log"
    phase="building the release app"
    echo "Building the release app (local ad-hoc signing; notarization is skipped)."
    if [ ! -d apps/web/node_modules ]; then
      npm --prefix apps/web ci
    fi
    # Fix the output directory so a caller's Cargo configuration cannot make
    # us install an old bundle. Tauri builds the UI before Cargo embeds it;
    # the desktop build script tracks dist so cached binaries cannot omit it.
    export CARGO_TARGET_DIR="$PWD/apps/desktop/src-tauri/target"
    # Like Jobsearch's family-build: a local signature, with no notarization.
    unset APPLE_API_ISSUER APPLE_API_KEY APPLE_API_KEY_PATH APPLE_ID APPLE_PASSWORD APPLE_TEAM_ID
    unset APPLE_CERTIFICATE APPLE_CERTIFICATE_PASSWORD
    export APPLE_SIGNING_IDENTITY="-"
    just stage-desktop-cli aarch64-apple-darwin release
    # Bundle the Python agent so the experimental runtime works in the
    # installed app without naming a path in Settings.
    just stage-python-agent
    (
      cd apps/desktop/src-tauri
      CI=true cargo tauri build --target aarch64-apple-darwin --bundles app \
        --config ../../../.dev/desktop-cli.json \
        --config ../../../.dev/desktop-python-agent.json
    )
    bundle="$CARGO_TARGET_DIR/aarch64-apple-darwin/release/bundle/macos/Hickory Docs.app"
    phase="verifying the release signature"
    echo "Verifying the release signature."
    codesign --verify --deep --strict "$bundle"
    destination="/Applications/Hickory Docs.app"
    # Copy completely before replacing the installed app; keep the previous
    # bundle until the replacement succeeds.
    phase="staging the app for installation"
    echo "Staging $destination."
    staging="$(mktemp -d /Applications/.hickory-install.XXXXXX)"
    trap 'rm -rf "$staging"' EXIT
    # Compile before replacing anything: no downloaded association utility.
    rustc --edition=2024 scripts/default-markdown-editor.rs -o "$staging/default-markdown-editor"
    ditto "$bundle" "$staging/Hickory Docs.app"
    phase="replacing the installed app"
    if [ -e "$destination" ]; then
      mv "$destination" "$staging/previous.app"
    fi
    if ! mv "$staging/Hickory Docs.app" "$destination"; then
      if [ -e "$staging/previous.app" ]; then
        mv "$staging/previous.app" "$destination"
      fi
      echo "Could not replace $destination; restored the previous app if present." >&2
      exit 1
    fi
    phase="registering the installed app with macOS"
    echo "Registering $destination with macOS."
    lsregister=/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister
    registration_status=registered
    # The ERR trap is inherited by command substitutions. Disable it in this
    # expected failure capture so the warning below is the only diagnostic.
    if ! registration_error="$(trap - ERR; "$lsregister" -f "$destination" 2>&1)"; then
      echo "$registration_error" >&2
      # -10822 is kLSServerCommunicationErr: the per-user LaunchServices
      # service is unavailable (for example, a remote/headless session), not
      # a malformed app. The app is already safely installed and Finder will
      # register it when that service returns.
      if [[ "$registration_error" == *"-10822"* ]]; then
        registration_status=pending
        echo "Warning: LaunchServices is unavailable; macOS will register $destination when its service returns." >&2
      else
        exit 1
      fi
    fi
    # Leave only the installed app for macOS to discover, after installation
    # succeeds. Unregister the build copy while its bundle still exists.
    # LaunchServices can refuse to scan this disposable copy (-10814).
    # That must not fail an already registered install or prevent removal.
    if ! "$lsregister" -u "$bundle"; then
      echo "Warning: macOS could not unregister the build copy; removing it anyway." >&2
    fi
    rm -rf "$bundle"
    if [ "$registration_status" = registered ]; then
      echo "Installed and registered $destination."
    else
      echo "Installed $destination; LaunchServices registration is pending."
    fi
    phase="setting the Markdown default"
    if ! "$staging/default-markdown-editor" "$destination" "$CARGO_TARGET_DIR"; then
      echo "Warning: Hickory Docs is installed, but its Markdown default could not be confirmed." >&2
      echo "In Finder, select a .md file → Get Info → Open with → choose $destination → Change All…" >&2
      echo "Default-editor diagnostics are in $log. No rebuild is needed to use the installed app." >&2
    fi
    echo "Installed $destination. Launch Hickory Docs from Applications or Spotlight."

# Run a hick document with the local executor.
run DOC *ARGS:
    cargo run -p hickory-cli -- run {{DOC}} {{ARGS}}

# Verify documents (drift/expectations). CI + pre-commit entry point.
verify DOC *ARGS:
    cargo run -p hickory-cli -- test {{DOC}} {{ARGS}}

# Run the agent on a prompt. Loads .env first, which is where API keys live —
# the CLI itself does not read .env, so calling `hick agent` directly needs
# the key already exported.
#   just agent "fix the failing test" --provider deepseek
agent PROMPT *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    set -a; [ -f .env ] && source .env; set +a
    cargo run -q -p hickory-cli -- agent "{{PROMPT}}" {{ARGS}}

# Rewrite stale hick:transform passages. Loads .env, same as `just agent`.
refresh DOC *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    set -a; [ -f .env ] && source .env; set +a
    cargo run -q -p hickory-cli -- refresh {{DOC}} {{ARGS}}

# Regenerate experiments/token-economics/report.md from the raw run JSONL.
tokens-report:
    cargo run -q -p hickory-agent --bin token_economics -- report

# Static token measurement of files via count_tokens (needs ANTHROPIC_API_KEY).
tokens-count *FILES:
    cargo run -q -p hickory-agent --bin token_economics -- count-tokens {{FILES}}

# ACP integration checks, including the process protocol and agent dock.
test-acp:
    cargo build -p hickory-cli --bin hick --example acp_fixture --example engine_client
    cargo test -p hickory-cli --lib serve::acp:: -- --test-threads=1
    cargo test -p hickory-cli --test serve_acp --test serve_agent --test byo_agent_surface --test workspace_fs -- --test-threads=1
    cargo test -p hickory-cli --test engine_lifecycle desktop_proxy_routes_acp -- --test-threads=1
    cd apps/web && npm run typecheck && npm test -- --maxWorkers=2

# Build the local CLI used by the ACP adapter smoke test.
build-cli:
    cargo build -p hickory-cli

# Uses the person's existing Codex login and spends model usage on a scratch document.
test-acp-live:
    cargo test -p hickory-cli --test serve_acp live_codex -- --ignored --nocapture

check-web:
    cd apps/web && npm run typecheck

# Headless-friendly workspace preview; prints its URL without opening a browser.
dev-spatial-mock:
    cd apps/web && npm run dev:mock

# Serve the real editor against an already-running scratch engine, without opening a browser.
dev-web api_origin:
    cd apps/web && HICKORY_API_ORIGIN={{quote(api_origin)}} npm run dev

test-spatial-browser url:
    cd apps/web && HICKORY_E2E_CHANNEL=chromium HICKORY_SPATIAL_URL={{quote(url)}} npx playwright test e2e/spatialWorkspace.spec.ts

test-files-browser url:
    cd apps/web && HICKORY_E2E_CHANNEL=chromium HICKORY_E2E_URL={{quote(url)}} npx playwright test e2e/files-editor.spec.ts

# Spatial layout geometry and native editor behavior.
test-spatial:
    cd apps/web && npm run typecheck && npm test -- src/shell/spatialTiles.test.ts src/shell/ribbonNavigation.test.ts src/shell/spatialLayout.test.ts src/shell/spatialDrawers.test.ts src/shell/SpatialShellView.test.tsx src/shell/SpatialController.test.ts src/editor/spatialExcerpt.test.ts src/views/SettingsView.test.tsx

# Drawer geometry and real editor state against a disposable native engine.
test-spatial-ui:
    bash scripts/test-spatial-ui.sh

# Focused command discovery and agent typing against a disposable native engine.
test-workflow-ui:
    bash scripts/test-spatial-ui.sh workflow-polish.spec.ts

test-workflow:
    cd apps/web && npm run typecheck && npm test -- src/App.test.tsx src/shell/CommandBar.test.tsx src/views/documentCommands.test.ts src/views/useWorkspaceAppearance.test.tsx src/views/ConversationEditor.test.tsx src/components/ChatDock.test.tsx src/editor/DocumentEditor.test.tsx src/components/CellPanel.test.tsx

test-debugger-paths:
    cargo test -p hick-dap --test project --test live_session -- --test-threads=1

# Regenerate the standalone contextual-tiling prototype from its source.
build-navigation-prototype:
    node work/navigation-prototype/build.cjs

# Check the standalone contextual-tiling prototype in Chromium.
check-navigation-prototype:
    node work/navigation-prototype/check.cjs

# Browser embedding and static JS/TS debugging, without a native engine.
install-browser-runtime:
    cd apps/web && npm install --save-exact js-interpreter@6.0.2 @jridgewell/trace-mapping@0.3.31

test-browser-core:
    cargo test -p hick-lang -p hick-lang-wasm
    cargo test -p hick-literate --test browser_literal_files
    cd apps/web && npm run typecheck && npm test -- src/embed src/debug/browser

test-browser-embedding *ARGS:
    cd apps/web && npx playwright test --config playwright.embed.config.ts {{ARGS}}

preview-browser-embedding:
    cd apps/web && npm run build:site && npx vite preview --config vite.site.config.ts --host 127.0.0.1

# Validate the agent UI without rerunning unrelated Rust suites.
test-agent-web:
    cd apps/web && npm run typecheck && npm test -- --maxWorkers=2

# Check all Rust test and example targets, matching CI.
clippy-all:
    cargo clippy --workspace --all-targets -- -D warnings

# Native FSKit frontend. Uses full Xcode without changing xcode-select globally.
build-fskit TARGET="":
    cargo run -p hickory-cli --example fskit_bundle -- {{TARGET}}

test-workspace-fs:
    cargo test -p hickory-collab
    cargo test -p hickory-cli --test workspace_fs -- --nocapture

# HICKORY_FSKIT_APP names a signed app with its workspace extension enabled.
# Uses a temporary workspace; does not open a window or use a model account.
test-fskit-live:
    cargo run -p hickory-cli --example fskit_live

# Desktop entry points live outside the root workspace.
check-desktop:
    cd apps/desktop/src-tauri && cargo fmt --all && cargo clippy --all-targets -- -D warnings

# Desktop PATH recovery and real-shell startup regression checks.
test-binary-discovery:
    cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --bin hickory-desktop launch_path::
    cargo test -p hick-term --lib shell_integration::
    cargo test -p hick-term --test a_real_shell_reports_its_directory --test typed_commands
    cargo test -p hickory-cli --lib serve::github::

# Real-process ownership, reconnect and shared-window regression checks.
test-engine:
    cargo build -p hickory-cli --bin hick --example engine_client --example acp_fixture
    cargo test -p hickory-cli --test engine_lifecycle --test up_loop --test up_stress --test serve_local -- --test-threads=1
    cargo test -p hickory-collab -p hickory-workspace -- --test-threads=1
    cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --test serves_one_origin -- --test-threads=1

# Agent conversation rendering and independently checked lineage evidence.
test-conversation-lineage:
    cargo test -p hick-literate --lib session_
    cargo test -p hickory-cli --test session_lens -- --test-threads=1
    cargo build -p hickory-cli --example acp_fixture
    cargo test -p hickory-cli --test serve_acp an_uncited_acp_answer -- --test-threads=1
    cd apps/web && npm run typecheck && npm test -- src/components/FeatureSettings.test.tsx src/components/ChatDock.test.tsx src/components/AcpControls.test.tsx src/views/SessionLens.test.tsx src/lib/conversationLineage.test.ts src/lib/lensSources.test.ts src/shell/linkShapes.test.ts src/lib/visibleElementRect.test.ts src/shell/ribbonNavigation.test.ts src/views/workspaceRibbonSources.test.ts src/lib/editorBufferTargets.test.ts src/lib/paths.test.ts src/editor/folding.test.ts src/views/ConversationEditor.test.tsx

# Compile the frontend bundled into desktop builds.
build-web:
    cd apps/web && npm run build

# Startup integration with the real editor, plus its layout and naming rules.
test-startup-web:
    cd apps/web && npm run typecheck && npm test -- src/App.test.tsx src/views/workspaceState.test.ts src/lib/newDoc.test.ts

# Editor bootstrap, guarded background initialization, and startup API retries.
test-startup:
    cargo build -p hickory-cli --bin hick --example engine_client
    cargo test -p hickory-cli --lib engine::startup::
    cargo test -p hickory-cli --test engine_lifecycle -- --test-threads=1
    cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --bin hickory-desktop launch_path::
    cd apps/web && npm run typecheck && npm test -- src/api/workspaceStartup.test.tsx src/api/client.test.ts src/App.test.tsx src/views/workspaceState.test.ts src/lib/newDoc.test.ts

test-startup-browser url:
    cd apps/web && HICKORY_E2E_CHANNEL=chromium HICKORY_E2E_URL={{quote(url)}} npx playwright test e2e/startup.spec.ts

# Make a locally signed macOS bundle without replacing the installed app.
build-desktop-app:
    just stage-desktop-cli "" dev
    just stage-python-agent
    cd apps/desktop/src-tauri && APPLE_SIGNING_IDENTITY="-" cargo tauri build --debug --bundles app --config ../../../.dev/desktop-cli.json --config ../../../.dev/desktop-python-agent.json
    codesign --verify --deep --strict "apps/desktop/src-tauri/target/debug/bundle/macos/Hickory Docs.app"

# Recent File/Folder persistence and workspace navigation.
test-recent-paths:
    cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --lib recent::tests
    cd apps/web && npm run typecheck && npm test -- src/views/useRecentFiles.test.tsx

# Agent context across open buffers, folder sessions, and folderless windows.
test-agent-context:
    cargo build -p hickory-cli --bin hick --example acp_fixture
    cargo test -p hickory-cli --test serve_agent --test serve_acp --test byo_agent_surface -- --test-threads=1
    cargo test -p hickory-agent -p hickory-collab --lib -- --test-threads=1
    cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --test serves_one_origin -- --test-threads=1
    just test-agent-web

# Folderless editor context and repository setup boundaries.
test-folderless:
    cargo test -p hickory-cli --test merge_driver_button --test serve_agent -- --test-threads=1
    cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --test serves_one_origin a_blank_window_has_editor_apis_but_no_open_folder -- --test-threads=1
    just test-agent-web

# Source-backed literate views, editable comparisons, and isolated visual bisect.
test-literate-views:
    cargo build -p hickory-cli --example engine_client --example acp_fixture
    cargo test -p hickory-cli --test literate_views -- --test-threads=1
    cargo test -p hickory-cli --test serve_acp acp_organizes_and_edits_a_disposable_view -- --test-threads=1
    cargo test -p hickory-cli --test engine_lifecycle -- --test-threads=1
    cd apps/web && npm test -- src/editor/comparison.test.ts src/lsp/representationMapping.test.ts src/lsp/cmLspFeatures.test.ts src/lsp/completion.test.ts

# Watched browser flows, with their own engine, state directory and scratch Git repository.
test-literate-editor FILTER="":
    cargo build -p hickory-cli --bin hick --example engine_client --example acp_fixture
    scripts/test-literate-editor.sh --grep {{quote(FILTER)}}

# Real patch-gutter Debug, automatic debugpy installation and native stepping.
test-unified-debug-ui:
    cargo build -p hickory-cli --bin hick --example engine_client --example acp_fixture
    HICKORY_E2E_DEBUG=1 scripts/test-literate-editor.sh --grep 'unified patch gutter|patch debugger keeps'

# Shared debugger rails, captured navigation, generated-source routing and keys.
test-unified-code-tools:
    cd apps/web && npm run typecheck && npm test -- --maxWorkers=2 src/debug/DebugRail.test.tsx src/debug/DebugStrip.test.tsx src/debug/plainDebugHosts.test.tsx src/debug/useDebugger.test.ts src/debug/GeneratedDebugTools.test.tsx src/debug/cmDebug.test.ts src/components/ComparisonTools.test.tsx src/lib/keymap.test.ts

test-source-breakpoints:
    cargo test -p hick-dap --lib --test project
    cargo test -p hickory-cli --lib serve::debug_bridge:: -- --test-threads=1
    cargo test -p hickory-cli --test debug_desktop source_breakpoints -- --test-threads=1
    just test-unified-code-tools

# Stage the self-contained Python agent runtime a desktop build bundles.
# Writes .dev/python-agent/runtime and the Tauri `files` config that ships it.
# Needs uv; see scripts/build-python-agent-runtime.sh for why the runtime is
# folded out of the locked environment rather than pointed at it.
stage-python-agent:
    ./scripts/build-python-agent-runtime.sh

# Stage the CLI sidecar for Tauri. Kept out of tauri.conf so cargo-only checks
# do not require an already-built binary. Every bundled build uses this config.
stage-desktop-cli TARGET="" PROFILE="release":
    #!/usr/bin/env bash
    set -euo pipefail
    target="{{TARGET}}"
    [ -n "$target" ] || target="$(rustc -vV | sed -n 's/^host: //p')"
    profile="{{PROFILE}}"
    host="$(rustc -vV | sed -n 's/^host: //p')"
    build_args=()
    output="$PWD/target"
    if [ "$target" != "$host" ]; then
      build_args=(--target "$target")
      output="$output/$target"
    fi
    CARGO_TARGET_DIR="$PWD/target" cargo build -p hickory-cli --bin hick --profile "$profile" ${build_args[@]+"${build_args[@]}"}
    suffix=""
    [[ "$target" != *windows* ]] || suffix=".exe"
    folder="$profile"
    [ "$profile" != dev ] || folder=debug
    mkdir -p .dev/binaries
    cp "$output/$folder/hick$suffix" ".dev/binaries/hick-$target$suffix"
    printf '%s\n' '{"bundle":{"externalBin":["../../../.dev/binaries/hick"]}}' > .dev/desktop-cli.json

# CLI launch, user PATH ownership, and Settings controls.
test-command-path:
    cargo test -p hickory-cli --bin hick cli_launch::
    cargo test -p hickory-cli --lib open_app::
    cargo test -p hickory-cli --test open_app
    cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --lib command_path::
    cd apps/web && npm run typecheck && npm test -- src/views/CommandPathSettings.test.tsx src/views/SettingsView.test.tsx

# Markdown pipe tables share the grid and keep Markdown on edit.
test-markdown-tables:
    cd apps/web && npm run typecheck && npm test -- src/components/MarkdownTable.test.tsx src/editor/markdownTables.test.tsx src/editor/rendered.test.ts src/editor/cards.test.ts

# Install the browsers used for browser embedding acceptance.
install-test-browsers *ARGS:
    cd apps/web && npx playwright install {{ARGS}} chromium firefox webkit

# Frontend regression suite after extracting the portable surface.
test-web:
    cd apps/web && npm run typecheck && npm test -- --maxWorkers=2

# Agent conversation controls, scroll following, and generated session formatting.
test-agent-pane:
    cargo test -p hickory-agent --lib -- --test-threads=1
    cd apps/web && npm run typecheck && npm test -- src/views/ConversationEditor.test.tsx src/components/ChatDock.test.tsx src/views/WorkspaceChat.test.tsx src/editor/session.test.tsx src/App.test.tsx

# Durable model-failure evidence, assumptions, review and historical receipts.
test-agent-audit:
    cargo test -p hickory-cli --lib serve::audit_evidence:: -- --test-threads=1
    cargo test -p hickory-agent --lib --test tools_edit_session -- --test-threads=1
    cargo test -p hickory-cli --test transform_tests --test session_lens --test serve_agent -- --test-threads=1
    cd apps/web && npm run typecheck && npm test -- --maxWorkers=2 src/elements/session/evidence.test.tsx src/components/AssumptionAudit.test.tsx src/components/AgentChanges.test.tsx src/views/ConversationEditor.test.tsx src/components/ChatDock.test.tsx

# Re-run UI cases that depend on asynchronous editor setup with bounded contention.
test-agent-audit-web:
    cd apps/web && npm run typecheck && npm test -- --maxWorkers=2 src/elements/session/evidence.test.tsx src/components/AssumptionAudit.test.tsx src/components/AgentChanges.test.tsx src/views/ConversationEditor.test.tsx src/components/ChatDock.test.tsx src/views/WorkspaceChat.test.tsx src/views/SessionLens.test.tsx src/App.test.tsx src/debug/browser/runtime.test.ts

# Full agent implementation, including historical external tool surfaces.
test-agent-implementation:
    cargo test -p hickory-agent -p hickory-cli --lib -- --test-threads=1
    cargo test -p hickory-cli --test agent_cell_vertex --test session_lens --test transform_tests -- --test-threads=1

# Exercise the locked Python worker and real OS confinement without a model key.
test-python-agent:
    scripts/check-python-agent.sh

# Real-browser Python setup, proposal review, history and cancellation.
test-python-agent-ui:
    scripts/test-python-agent-ui.sh

# Validate the local guarantee-maintainer skill metadata.
validate-guarantee-skill SKILL_CREATOR_DIR:
    uv run --with pyyaml python "{{SKILL_CREATOR_DIR}}/scripts/quick_validate.py" .codex/skills/guarantee-maintainer

# File menu, independent unnamed buffers, and opening selected files in place.
test-file-menu:
    cd apps/web && npm run typecheck && npm test -- src/App.test.tsx src/views/workspaceState.test.ts src/views/folderPane.test.tsx src/views/useUntitledSave.test.tsx src/views/useUnsavedLifecycle.test.tsx src/views/workspaceTabs.test.tsx src/router.test.ts
    cargo test -p hickory-cli --test open_selected_file -- --test-threads=1

# Attaching the first folder preserves this window and its existing editors.
test-first-folder:
    cd apps/desktop/src-tauri && cargo test --test first_folder --test serves_one_origin -- --test-threads=1
    cd apps/web && npm test -- src/views/folderPane.test.tsx

# Portable changeset replay, exact inventory, revision inputs and tool isolation.
test-literate-projects:
    cargo test -p hick-lang --test project
    cargo test -p hick-lsp --test project
    cargo test -p hick-dap --test project
    cargo test -p hick-literate --test project
    cargo test -p hickory-cli --test project_revision

check-literate-projects:
    cargo check -p hickory-cli -p hick-lang-wasm

# Shared patch view and the real WASM project evaluator.
test-literate-projects-web:
    cd apps/web && npm run typecheck
    cd apps/web && npm test -- src/elements/patch/view.test.tsx --maxWorkers=2

# Existing parser, coordinate-map, element and debugger contracts near this change.
test-literate-projects-regressions:
    just test-literate-projects-native-regressions
    cd apps/web && npm test -- --maxWorkers=2

test-literate-projects-native-regressions:
    cargo test -p hick-lang --lib
    cargo test -p hick-lsp --lib
    cargo test -p hick-dap --lib
    cargo test -p hickory-cli --lib serve::refactor

lint-literate-projects:
    cargo clippy -p hick-lang -p hick-lang-wasm -p hick-lsp -p hick-dap -p hick-literate -p hickory-cli --all-targets -- -D warnings

# Explicit native acceptance. Downloads the pinned adapter into the local cache.
install-literate-project-debugger:
    cargo run -p hickory-cli -- dap install csharp

# This is separate from pure source-equivalence checks and requires .NET/NuGet.
test-literate-projects-csharp:
    cargo test -p hickory-cli --test literate_project_csharp -- --ignored --nocapture

# A real read-only Mercurial provider check; this CLI is a test tool, not linked.
test-literate-projects-hg:
    python3 -m venv .hick-cache/test-hg
    .hick-cache/test-hg/bin/python -m pip install --disable-pip-version-check mercurial==7.0
    PATH="$PWD/.hick-cache/test-hg/bin:$PATH" cargo test -p hickory-cli --test project_revision mercurial_revision -- --ignored --nocapture

# Regenerate the executable fixtures' expected source trees without running cells.
regenerate-literate-project-fixtures:
    cargo run -p hickory-cli -- weave crates/hick-lang/tests/fixtures/literate-project/project.md
    cargo run -p hickory-cli -- weave crates/hick-lang/tests/fixtures/efcore-project/project.md

# Exact staged verification for a maintained project selected by its entry/root.
verify-literate-project-index ENTRY ROOT=".":
    cargo run -p hickory-cli -- project verify {{ENTRY}} --root {{ROOT}} --revision :index

# Run all browser unit tests after element/parser changes.
test-literate-projects-web-regressions:
    cd apps/web && npm run typecheck
    cd apps/web && npm test -- --maxWorkers=2

# Verify declared entry routes and expected fixture outputs without execution.
verify-literate-project-fixtures:
    cargo run -p hickory-cli -- lint crates/hick-lang/tests/fixtures/literate-project/project.md crates/hick-lang/tests/fixtures/literate-project/library.md crates/hick-lang/tests/fixtures/efcore-project/project.md crates/hick-lang/tests/fixtures/efcore-project/persistence.md
    cargo run -p hickory-cli -- test crates/hick-lang/tests/fixtures/literate-project/project.md crates/hick-lang/tests/fixtures/literate-project/library.md crates/hick-lang/tests/fixtures/efcore-project/project.md crates/hick-lang/tests/fixtures/efcore-project/persistence.md

# Common editor contracts and portable browser patch regeneration.
test-literate-comparison-editor:
    cd apps/web && npm run typecheck
    cd apps/web && npm test -- src/lib/projectComparison.test.ts src/components/ComparisonEditor.test.tsx src/components/ComparisonTools.test.tsx src/debug/documentDebugContext.test.ts src/elements/patch/Editor.test.tsx src/views/workspaceTabs.test.tsx --maxWorkers=2

# All portable projects declared in the exact candidate index; no working reads.
verify-literate-projects-index:
    cargo run -p hickory-cli -- project verify-index

# Real HTTP capture/finalization and existing-queue source incorporation.
test-literate-agent-projects:
    cargo build -p hickory-cli --example acp_fixture
    cargo test -p hickory-cli --test literate_agent_projects -- --nocapture
    cargo test -p hickory-agent --test tools_edit_session
    cargo test -p hickory-collab

# Real worktree routing, snapshot guards and per-target undo.
test-literate-worktrees:
    cargo test -p hickory-cli --test merged_view
    cd apps/web && npm test -- src/views/MergedView.test.tsx --maxWorkers=2

# Compact editor chrome, preferences and native browser geometry.
test-editor-density:
    cd apps/web && npm run typecheck && npm test -- src/views/workspaceTabs.test.tsx src/views/SettingsView.test.tsx src/shell/CommandBar.test.tsx src/shell/PaneHeader.test.tsx src/components/PlainFilePane.test.tsx src/editor/EditorRuler.test.tsx src/shell/SpatialShellView.test.tsx src/lib/keymap.test.ts src/lib/cardRail.test.ts --maxWorkers=2

test-editor-density-ui:
    bash scripts/test-spatial-ui.sh editor-density.spec.ts

# Files text rows, floating preview surfaces and themed hints.
test-files-previews:
    cd apps/web && npm run typecheck && npm test -- src/shell/SpatialController.test.ts src/shell/spatialDrawers.test.ts src/shell/FilesystemTreeEditor.test.tsx src/lib/tooltip.test.ts --maxWorkers=2

# Reproduce bypasses against a disposable copy of an existing walkthrough.
test-spatial-walkthrough project:
    HICKORY_SPATIAL_PROJECT={{quote(project)}} bash scripts/test-spatial-ui.sh spatialWalkthrough.spec.ts

# Resize the actual Files pane across its preview-layout breakpoint.
test-responsive-files-ui:
    bash scripts/test-spatial-ui.sh spatialWorkspace.spec.ts --grep 'resizing Files'

# Install the frontend dependencies from the checked-in lockfile.
setup-web:
    cd apps/web && npm ci

# Folder-scoped open editors, layout restoration and close flushing.
test-folder-window-web:
    just check-web
    cd apps/web && npm test -- --maxWorkers=2 src/views/useWorkspaceUi.test.tsx src/views/restoreFolderLayout.test.ts src/shell/SpatialShellView.test.tsx src/shell/spatialLayout.test.ts src/views/folderPane.test.tsx src/views/useUnsavedLifecycle.test.tsx src/views/useUntitledSave.test.tsx src/views/workspaceTabs.test.tsx src/lib/uiState.test.ts src/App.test.tsx

test-folder-window-state:
    just test-folder-window-web
    just build-web
    cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --test serves_one_origin folder_layout -- --test-threads=1

# Literate debugger navigation and exact source-position mapping.
test-debug-navigation:
    cd apps/web && npm test -- src/debug/literateDebugNavigation.test.tsx src/lib/capturedContexts.test.ts src/debug/projectFrameOffset.test.ts src/debug/projectDebugSurface.test.tsx src/debug/DebugRail.test.tsx src/debug/preferredDebugLocation.test.ts src/debug/plainDebugHosts.test.tsx src/components/ComparisonTools.test.tsx src/lsp/representationMapping.test.ts --maxWorkers=2

# File tabs and Files rows share context-menu actions and prompts.
test-file-tab-menu:
    just check-web
    cd apps/web && npm test -- src/views/useFileTabMenu.test.tsx src/shell/FolderTreePane.test.tsx src/shell/SpatialShellView.test.tsx --maxWorkers=2

# Exact-source project selection, breakpoint rails and frozen comparison tools.
test-exact-source-debug:
    cargo test -p hick-lang --test project_debug --test project
    cargo test -p hick-lsp --test project
    cd apps/web && npm run typecheck && npm test -- --maxWorkers=2 src/debug/documentDebugContext.test.ts src/debug/DebugRail.test.tsx src/debug/useDebugger.test.ts src/components/ComparisonTools.test.tsx src/views/workspaceTabs.test.tsx

# Real native adapter: file-block, Old and New launches preserve displayed bytes.
test-exact-source-debug-ui:
    cargo build -p hickory-cli --bin hick --example engine_client --example acp_fixture
    HICKORY_E2E_DEBUG=1 scripts/test-literate-editor.sh --grep 'exact-source debug|unified patch gutter|patch debugger keeps'

# Launch discovery/configuration, atomic project versions and profile chooser.
test-launch-profiles:
    cargo test -p hick-lang --test project_debug --test project
    cargo test -p hickory-cli --test launch_profiles
    cargo test -p hickory-cli --lib serve::test_run
    cd apps/web && npm run typecheck && npm test -- --maxWorkers=2 src/debug/launchProfiles.test.tsx src/debug/projectDebugSurface.test.tsx src/debug/GeneratedDebugTools.test.tsx src/debug/useDebugger.test.ts src/debug/plainDebugHosts.test.tsx src/debug/DebugRail.test.tsx src/components/ComparisonTools.test.tsx

# Real app/profile selection, both atomic readings, library breakpoints and Run.
test-launch-profiles-ui:
    cargo build -p hickory-cli --bin hick --example engine_client --example acp_fixture
    HICKORY_E2E_DEBUG=1 scripts/test-literate-editor.sh --grep 'launch profiles|unified patch gutter|patch debugger keeps|exact-source debug'

# Negotiated ACP changeset completion and the shared MCP verification tools.
test-agent-changesets:
    cargo build -p hickory-cli --example acp_fixture
    cargo test -p hickory-cli --lib changeset:: -- --nocapture
    cargo test -p hickory-cli --lib workspace_capture:: -- --nocapture
    cargo test -p hickory-cli --lib serve::acp::edits:: -- --nocapture
    cargo test -p hickory-cli --test agent_changesets -- --nocapture
    cargo test -p hickory-cli --test literate_agent_projects -- --nocapture

# Final artifact rendering uses the same patch and native-tool context as notes.
test-agent-changesets-web:
    cd apps/web && npm run typecheck && npm test -- --maxWorkers=2 src/components/ChangesetDocument.test.tsx src/elements/patch/Editor.test.tsx src/elements/patch/view.test.tsx src/components/ComparisonTools.test.tsx src/components/AgentChanges.test.tsx src/views/ConversationEditor.test.tsx src/components/ChatDock.test.tsx src/views/WorkspaceChat.test.tsx

check-agent-changesets:
    cargo fmt --all -- --check
    just check-file-length
    cargo clippy -p hickory-cli --lib --example acp_fixture --test agent_changesets -- -D warnings

# Provenance visualization grouping and overlay geometry.
test-ribbon-grouping:
    cd apps/web && npm test -- src/lib/coalesceRibbons.test.ts src/shell/coalesceLinks.test.ts src/lib/ribbons.test.ts src/shell/linkShapes.test.ts src/lib/ribbonGeometry.test.ts

# Real editor, fresh-context exchange and ACP repair, with and without a folder.
test-workspace-changesets-browser:
    cargo build -p hickory-cli --example acp_fixture
    cargo test -p hickory-cli --test agent_changesets browser_workspace_changesets -- --ignored --nocapture

# Personal agent response recovery and the actual durable store.
test-response-recovery:
    just check-web
    cd apps/web && npm test -- --maxWorkers=2 src/lib/responseRecovery.test.ts src/components/ChatDock.test.tsx src/views/WorkspaceChat.test.tsx src/views/ConversationEditor.test.tsx src/views/useUnsavedLifecycle.test.tsx src/App.test.tsx
    cargo test -p hickory-workspace --lib
    cargo check -p hickory-cli

test-response-recovery-ui:
    bash scripts/test-spatial-ui.sh responseRecovery.spec.ts

check-response-recovery:
    cargo fmt --all -- --check
    cargo clippy -p hickory-workspace -p hickory-cli --lib -- -D warnings
    just check-file-length

# Shared Rust/Python context contract only; no provider or broad regression checks.
test-context-protocol:
    python3 scripts/generate-context-protocol.py --check
    cargo test -p hick-context-protocol --test conformance
    uv run --project apps/agent --locked --no-editable --reinstall-package hickory-agent-worker python -m unittest discover -s apps/agent/tests -p test_context_protocol.py

# Immutable context storage and real filesystem/editor adapter witnesses (#49).
test-context-store:
    cargo test -p hick-context-store --test lineage
    cargo test -p hickory-agent --lib tools::read_file::tests
    cargo test -p hickory-cli --lib serve::acp::read_receipt::

# Bootstrap the native adapter from this repository's generated Rust source.
literate-bootstrap:
    CARGO_TARGET_DIR="{{justfile_directory()}}/target/dogfood" cargo build --locked --release --manifest-path hickory-dogfood/tools/behavior-engine/Cargo.toml

# Materialize the application and maintenance outputs from maintained Markdown.
literate-generate:
    python3 hickory-dogfood/tools/behavior.py generate

# Deterministic syntax-based review scorecard; arguments pass to the analyzer.
review-complexity *ARGS:
    python3 scripts/review-complexity.py {{ARGS}}

# Focused structural invariance, revision and source-location witnesses.
test-review-complexity:
    python3 -m unittest discover -s scripts/tests -p test_review_complexity.py -v

# Read-only source/output agreement and editorial accounting checks.
literate-check:
    python3 scripts/generate-api.py --check
    python3 hickory-dogfood/tools/behavior.py check
    python3 hickory-dogfood/tools/behavior.py audit
</maint:entry>
</maint:doc>
