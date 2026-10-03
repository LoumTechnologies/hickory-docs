# Hickory Docs task runner — the only entry point for project tasks.

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

# Confined tool installers and the diagnostics returned to the app.
test-tool-install:
    cargo test -p hickory-executor-sandbox --test installer_home
    cargo test -p hickory-cli --lib tool_install::tests
    cargo test -p hick-dap --test live_session -- --test-threads=1
    cargo build -p hickory-cli

# Regenerate everything derived from another file in this repo.
#
# Today that is one thing: the web app's language table, emitted from
# `hick-lsp`'s routing table. It exists because the two used to be written
# down separately and drifted apart in both directions at once — nineteen
# extensions' worth — which is the same class of bug that has made a language
# server unreachable here twice before.
codegen:
    scripts/check-codegen.sh --write

# Fail if anything generated is out of date with its source. Runs in CI and in
# the pre-commit hook, so drift cannot reach master.
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
    if [ "$(uname -s)" = Darwin ] && [ "$(sw_vers -productVersion | cut -d. -f1)" -ge 26 ]; then just build-fskit; fi
    just check-codegen
    just check-file-length
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo build --workspace
    cargo build -p hickory-cli --example acp_fixture
    cargo test --workspace -- --test-threads=1
    cd apps/web && npm run typecheck && npm test

# Build the static marketing site into apps/web/dist — no server, no accounts,
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
    VITE_POSTHOG_KEY="$key" npm run build:site
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

# Build and install the native macOS app in Applications (no disk image needed).
local-install:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "$(uname -s)" != Darwin ] || [ "$(uname -m)" != arm64 ]; then
      echo "local-install requires an Apple Silicon Mac." >&2
      exit 1
    fi
    if ! command -v cargo-tauri >/dev/null 2>&1; then
      echo "Install the desktop build tool first: cargo install tauri-cli --version '^2' --locked" >&2
      exit 1
    fi
    if [ ! -w /Applications ]; then
      echo "local-install needs write access to /Applications." >&2
      exit 1
    fi
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
    (
      cd apps/desktop/src-tauri
      CI=true cargo tauri build --target aarch64-apple-darwin --bundles app
    )
    bundle="$CARGO_TARGET_DIR/aarch64-apple-darwin/release/bundle/macos/Hickory Docs.app"
    codesign --verify --deep --strict "$bundle"
    destination="/Applications/Hickory Docs.app"
    # Copy completely before replacing the installed app; keep the previous
    # bundle until the replacement succeeds.
    staging="$(mktemp -d /Applications/.hickory-install.XXXXXX)"
    trap 'rm -rf "$staging"' EXIT
    # Compile before replacing anything: no downloaded association utility.
    rustc --edition=2024 scripts/default-markdown-editor.rs -o "$staging/default-markdown-editor"
    ditto "$bundle" "$staging/Hickory Docs.app"
    if [ -e "$destination" ]; then
      mv "$destination" "$staging/previous.app"
    fi
    if ! mv "$staging/Hickory Docs.app" "$destination"; then
      if [ -e "$staging/previous.app" ]; then
        mv "$staging/previous.app" "$destination"
      fi
      exit 1
    fi
    /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "$destination"
    # Leave only the installed app for macOS to discover, after installation
    # succeeds. Unregister the build copy while its bundle still exists.
    # LaunchServices can refuse to scan this disposable copy (-10814).
    # That must not fail an already registered install or prevent removal.
    if ! /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -u "$bundle"; then
      echo "Warning: macOS could not unregister the build copy; removing it anyway." >&2
    fi
    rm -rf "$bundle"
    "$staging/default-markdown-editor" "$destination"
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

# Run one token-economics experiment (needs ANTHROPIC_API_KEY).
tokens-run SPEC:
    cargo run -q -p hickory-agent --bin token_economics -- run {{SPEC}}

# Static token measurement of files via count_tokens (needs ANTHROPIC_API_KEY).
tokens-count *FILES:
    cargo run -q -p hickory-agent --bin token_economics -- count-tokens {{FILES}}

# ACP integration checks, including the process protocol and agent dock.
test-acp:
    cargo build -p hickory-cli --bin hick --example acp_fixture --example engine_client
    cargo test -p hickory-cli --lib serve::acp:: -- --test-threads=1
    cargo test -p hickory-cli --test serve_acp --test serve_agent --test byo_agent_surface --test workspace_fs -- --test-threads=1
    cargo test -p hickory-cli --test engine_lifecycle desktop_proxy_routes_acp -- --test-threads=1
    cd apps/web && npm run typecheck && npm test

# Build the local CLI used by the ACP adapter smoke test.
build-cli:
    cargo build -p hickory-cli

# Uses the person's existing Codex login and spends model usage on a scratch document.
test-acp-live:
    cargo test -p hickory-cli --test serve_acp live_codex -- --ignored --nocapture

check-web:
    cd apps/web && npm run typecheck

# Validate the agent UI without rerunning unrelated Rust suites.
test-agent-web:
    cd apps/web && npm run typecheck && npm test

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
    cargo test -p hick-literate --lib session_elements::tests
    cargo test -p hickory-cli --test session_lens -- --test-threads=1
    cd apps/web && npm run typecheck && npm test -- src/components/FeatureSettings.test.tsx src/components/ChatDock.test.tsx src/components/AcpControls.test.tsx src/views/SessionLens.test.tsx src/lib/conversationLineage.test.ts src/lib/lensSources.test.ts

# Compile the frontend bundled into desktop builds.
build-web:
    cd apps/web && npm run build

# Startup integration with the real editor, plus its layout and naming rules.
test-startup-web:
    cd apps/web && npm run typecheck && npm test -- src/App.test.tsx src/views/workspaceState.test.ts src/lib/newDoc.test.ts

# Make a locally signed macOS bundle without replacing the installed app.
build-desktop-app:
    cd apps/desktop/src-tauri && APPLE_SIGNING_IDENTITY="-" cargo tauri build --debug --bundles app
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

# Source-backed literate views, editable comparisons, and isolated visual bisect.
test-literate-views:
    cargo build -p hickory-cli --example engine_client --example acp_fixture
    cargo test -p hickory-cli --test literate_views -- --test-threads=1
    cargo test -p hickory-cli --test serve_acp acp_organizes_and_edits_a_disposable_view -- --test-threads=1
    cargo test -p hickory-cli --test engine_lifecycle -- --test-threads=1
    cd apps/web && npm test -- src/editor/comparison.test.ts src/lsp/representationMapping.test.ts src/lsp/cmLspFeatures.test.ts src/lsp/completion.test.ts

# Watched browser flows, with their own engine, state directory and scratch Git repository.
test-literate-editor:
    cargo build -p hickory-cli --example engine_client
    scripts/test-literate-editor.sh
