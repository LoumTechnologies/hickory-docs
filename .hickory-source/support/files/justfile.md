<maint:doc xmlns:maint="http://www.hickorydocs.com/1.0">
# Maintain justfile

For: maintainers changing the literate workflow and reviewers checking migration evidence.

Edit this entry, regenerate, and commit its output with the Markdown. Historical evidence retains its original scope.

The local installer can finish an already built bundle with `local-install true`.
Its retry diagnostic quotes the checkout path for the shell and asks sudo to
validate credentials before rerunning only installation. The retry verifies the
signature again and elevates only staging-directory creation, ownership setup,
replacement, rollback and cleanup. Building and per-user macOS registration stay
with the caller. If rollback itself fails, retain the previous bundle and report
its location instead of deleting the only recoverable copy.

`test-static-source` is the focused checkpoint for the bulk index, provenance projection, installed CLI artifacts and read-only viewer. `site` exports the declared project through the same product command, installs only frozen npm dependencies for analysis, and supplies an explicit public source-link prefix. Other languages retain their registered syntax grammars without starting an LSP by default.


`native-debug-smoke apple/android` is an additive scoped real-system acceptance
command. It preserves `simple-test` and all existing recipe names/coverage. Its
thin recipe bootstraps the dependency-free Rust preparation adapter, which reuses
the existing four-flow native fixture runner and installs only missing pinned
project-local dependencies. The [native debugger reading](../../frontends/scripts/test-native-debug.py.md)
records machine SDK requirements, tool/cache ownership, caller inventory and
verification limits. This smoke does not claim device input or accessibility.

The Simple dev recipe opts the UI and engine into build identity together.
Packaging recipes omit the flag, including those that compile a debug host
or APK for distribution.
Keep Cargo's build and lock status visible for macOS launch and reload, before
the Rust launcher can report which native startup step it is running.

The Apple build and test recipes select full Xcode before invoking SwiftPM or
Xcode build tools. SwiftUI's macro plugins come from full Xcode. When the
Command Line Tools are selected globally, look for a full installation under
`/Applications` and set `DEVELOPER_DIR` only for the recipe. An explicit
`DEVELOPER_DIR` keeps the caller in control of which installation is used.

<maint:entry path="justfile">
# Hickory Docs task runner — the only entry point for project tasks.

# Configure the optional checks provider; default workload is just check.
[positional-arguments]
checks-configure executable *COMMAND:
    cargo run --quiet -p hickory-cli --bin hickory-checks -- configure "$@"

# Read normalized provider status without starting a service or workload.
checks-provider-status:
    cargo run --quiet -p hickory-cli --bin hickory-checks -- status

# Submit a detached local-ci run; an optional full commit ID uses its Git adapter.
checks-submit revision="":
    cargo run --quiet -p hickory-cli --bin hickory-checks -- run {{ if revision == "" { "" } else { quote(revision) } }}

# Select clone-local post-commit behavior: local (default), local-ci, or off.
checks-hook mode:
    @case {{quote(mode)}} in local|local-ci|off) git config --local hickory.postCommitChecks {{quote(mode)}} ;; *) echo 'Choose local, local-ci, or off' >&2; exit 1 ;; esac

# Focused checks-provider and UI safety witnesses; no broad regression suite.
checks-integration-test provider="":
    cargo test -p hickory-cli --lib checks::tests
    cd apps/web && npx vitest run src/views/ChecksPane.test.tsx
    {{ if provider == "" { "true" } else { "cargo build -p hickory-cli --bin hickory-checks --example checks_fixture && python3 scripts/test-checks-integration.py target/debug/hickory-checks " + quote(provider) + " target/debug/examples/checks_fixture" } }}

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
    python3 -m unittest discover -s scripts/tests -p test_dev_supervisor.py -v
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

# Homepage hot reload and commit restarts, separate from the desktop app.
dev-site: _dev-supervisor
    target/dev-supervisor --web-once dev:site

# Check only the bulk-index/static-export behavior before publishing it.
test-static-source:
    #!/usr/bin/env bash
    set -euo pipefail
    npm ci --prefix code-models/typescript --no-fund --no-audit
    cargo build --locked -p hickory-cli --bin hick
    npm --prefix code-models/typescript test
    python3 scripts/test-map-source-index.py
    python3 scripts/test-capture-source-language.py
    HICKORY_EXPORT_HICK="$PWD/target/debug/hick" python3 scripts/test-export-source-site.py
    npm --prefix apps/web test -- src/editor/paragraphReading.test.ts src/lsp/symbolInspection.test.ts src/embed/staticLanguage.test.ts src/embed/SourcePage.test.tsx src/embed/SourceFileTree.test.tsx src/embed/staticArtifacts.test.ts src/embed/sourceLinks.test.ts

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
    npm ci --prefix code-models/typescript --no-fund --no-audit
    cargo build --locked -p hickory-cli --bin hick
    target/debug/hick project export project.md --output apps/web/public/source --model typescript=code-models/typescript/hick-model-typescript --install-dependencies --source-url "https://github.com/LoumTechnologies/hickory-docs/blob/{revision}/"
    cd apps/web
    HICKORY_SITE_ONLY=1 VITE_POSTHOG_KEY="$key" npm run build:site
    echo
    echo "Static site built: apps/web/dist-site"
    echo "It needs no backend. The landing page and its demos run in the browser."
    echo "Serves /install.sh — note that the installer pulls from GitHub releases,"
    echo "which strangers can only reach once the repository is public."

# macOS uses AppKit. Other platforms default to the Simple browser.
dev platform="":
    #!/usr/bin/env bash
    set -euo pipefail
    platform={{quote(platform)}}
    if [ -z "$platform" ]; then
      if [ "$(uname -s)" = Darwin ]; then platform=apple; else platform=browser; fi
    fi
    if [ -z "${HICKORY_PROJECT_DIR:-}" ]; then
      just dev-seed
      export HICKORY_PROJECT_DIR="$PWD/.dev/project"
    fi
    verifier="$(git config --get hickory.projectVerifier || true)"
    if [ "$(git config --get core.hooksPath || true)" != .githooks ] ||
       [ "$(git config --get hickory.projectEntry || true)" != project.md ] ||
       [ ! -x "$verifier" ]; then
      cargo build --locked -p hickory-cli --bin hick
      git config --local core.hooksPath .githooks
      target/debug/hick project install-hook project.md
    fi
    exec just simple-dev "$platform"

# Run the Simple workspace against an isolated local engine.
test-e2e:
    cargo build --locked -p hickory-cli --example engine_client
    python3 scripts/test-simple-browser.py --early-source-only
    python3 scripts/test-simple-browser.py --agent-only

# Signal the current checkout's development supervisor.
dev-stop: _dev-supervisor
    target/dev-supervisor --desktop-stop stop

dev-clean: _dev-supervisor
    target/dev-supervisor --desktop-stop clean

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

# Build and install the Simple native macOS app in Applications.
local-install:
    #!/usr/bin/env bash
    set -euo pipefail
    destination="/Applications/Hickory Simple.app"
    staging="$(mktemp -d /Applications/.hickory-simple-install.XXXXXX)"
    trap 'rmdir "$staging" 2>/dev/null || true' EXIT
    just build-desktop-app Debug
    ditto "target/simple-app/Debug/Hickory Simple.app" "$staging/Hickory Simple.app"
    codesign --verify --deep --strict "$staging/Hickory Simple.app"
    if [ -e "$destination" ]; then mv "$destination" "$staging/previous.app"; fi
    if ! mv "$staging/Hickory Simple.app" "$destination"; then
      if [ -d "$staging/previous.app" ]; then mv "$staging/previous.app" "$destination"; fi
      exit 1
    fi
    if [ -d "$staging/previous.app" ]; then trash "$staging/previous.app"; fi
    echo "Installed $destination"

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

# Focused in-app feature measurement witnesses; no paid provider or broad regression.
test-feature-measurements:
    cargo build --offline --locked --release -p hickory-cli --example acp_fixture
    cargo test --offline --locked --release -p hickory-cli --lib serve::acp::experiments::tests
    cargo test --offline --locked --release -p hickory-cli --test feature_experiments
    cd apps/web && npm test -- src/components/FeatureExperiments.test.tsx --maxWorkers=2
    cd apps/web && npm test -- src/components/ChatDock.test.tsx --testNamePattern "detects adapters|measures only" --maxWorkers=2

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


# Editor preview against the supplied engine; commits restart only this frontend.
dev-web api_origin: _dev-supervisor
    target/dev-supervisor --web-once dev {{quote(api_origin)}}


# Spatial layout geometry and native editor behavior.
test-spatial:
    cd apps/web && npm run typecheck && npm test -- src/shell/spatialTiles.test.ts src/shell/spatialLayout.test.ts src/shell/spatialDrawers.test.ts src/shell/SpatialController.test.ts src/editor/spatialExcerpt.test.ts


test-workflow:
    cd apps/web && npm run typecheck && npm test -- src/views/ConversationEditor.test.tsx src/components/ChatDock.test.tsx src/components/CellPanel.test.tsx

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


# Desktop PATH recovery and real-shell startup regression checks.
test-binary-discovery:
    cargo test -p hick-term --lib shell_integration::
    cargo test -p hick-term --test a_real_shell_reports_its_directory --test typed_commands
    cargo test -p hickory-cli --lib serve::github::

# Real-process ownership, reconnect and shared-window regression checks.
test-engine:
    cargo build -p hickory-cli --bin hick --example engine_client --example acp_fixture
    cargo test -p hickory-cli --test engine_lifecycle --test up_loop --test up_stress --test serve_local -- --test-threads=1
    cargo test -p hickory-collab -p hickory-workspace -- --test-threads=1

# Agent conversation rendering and independently checked lineage evidence.
test-conversation-lineage:
    cargo test -p hick-literate --lib session_
    cargo test -p hickory-cli --test session_lens -- --test-threads=1
    cargo build -p hickory-cli --example acp_fixture
    cargo test -p hickory-cli --test serve_acp an_uncited_acp_answer -- --test-threads=1
    cd apps/web && npm run typecheck && npm test -- src/components/FeatureSettings.test.tsx src/components/ChatDock.test.tsx src/components/AcpControls.test.tsx src/views/SessionLens.test.tsx src/lib/conversationLineage.test.ts src/lib/lensSources.test.ts src/shell/linkShapes.test.ts src/lib/visibleElementRect.test.ts src/lib/paths.test.ts src/views/ConversationEditor.test.tsx

# Build the Simple browser editor.
build-web:
    cd apps/web && npm run build


# Editor bootstrap, guarded background initialization, and startup API retries.
test-startup:
    cargo build -p hickory-cli --bin hick --example engine_client
    cargo test -p hickory-cli --lib engine::startup::
    cargo test -p hickory-cli --test engine_lifecycle -- --test-threads=1
    cd apps/web && npm run typecheck && npm test -- src/api/client.test.ts


# Resolve full Xcode without changing the machine's toolchain selection.
_xcode-developer-dir:
    #!/usr/bin/env bash
    set -euo pipefail
    if xcodebuild -version >/dev/null 2>&1; then
      exec xcode-select -p
    fi
    if [ -z "${DEVELOPER_DIR:-}" ]; then
      for developer in /Applications/Xcode*.app/Contents/Developer; do
        if [ -x "$developer/usr/bin/xcodebuild" ]; then
          export DEVELOPER_DIR="$developer"
        fi
      done
      if [ -n "${DEVELOPER_DIR:-}" ] && xcodebuild -version >/dev/null 2>&1; then
        exec xcode-select -p
      fi
    fi
    echo 'A working full Xcode installation is required. Install Xcode and set DEVELOPER_DIR to its Contents/Developer directory.' >&2
    exit 2

# Stage the native app, workspace host and shared editor assets together.
build-desktop-app configuration="Debug":
    #!/usr/bin/env bash
    set -euo pipefail
    configuration={{quote(configuration)}}
    case "$configuration" in
      Debug) cargo_profile=dev; cargo_directory=debug ;;
      Release) cargo_profile=release; cargo_directory=release ;;
      *) echo 'Choose Debug or Release.' >&2; exit 2 ;;
    esac
    if [ "$(uname -s)" != Darwin ]; then echo 'Build this app on macOS with Xcode.' >&2; exit 2; fi
    DEVELOPER_DIR="$(just _xcode-developer-dir)"
    export DEVELOPER_DIR
    if [ ! -x apps/web/node_modules/.bin/vite ] || [ apps/web/package-lock.json -nt apps/web/node_modules ]; then
      npm --prefix apps/web ci
    fi
    just simple-build browser
    cargo build --locked --target-dir "$PWD/target" --profile "$cargo_profile" -p hickory-cli --example simple_dev
    xcodegen generate --spec apps/apple-simple/project.yml
    build="$PWD/target/simple-app"
    xcodebuild -quiet -jobs 2 -project apps/apple-simple/HickorySimple.xcodeproj       -target HickorySimpleMac -configuration "$configuration" ARCHS="$(uname -m)"       SYMROOT="$build" OBJROOT="$build/obj" CODE_SIGNING_ALLOWED=NO build
    app="$build/$configuration/Hickory Simple.app"
    ditto "$build/$configuration/HickorySimpleMac.app" "$app"
    install -m 0755 "target/$cargo_directory/examples/simple_dev" "$app/Contents/MacOS/hickory-simple-host"
    ditto apps/web/dist-simple "$app/Contents/Resources/HickoryWeb"
    codesign --force --sign - "$app/Contents/MacOS/hickory-simple-host"
    codesign --force --sign - "$app"
    codesign --verify --deep --strict "$app"


# Agent context across open buffers, folder sessions, and folderless windows.
test-agent-context:
    cargo build -p hickory-cli --bin hick --example acp_fixture
    cargo test -p hickory-cli --test serve_agent --test serve_acp --test byo_agent_surface -- --test-threads=1
    cargo test -p hickory-agent -p hickory-collab --lib -- --test-threads=1
    just test-agent-web


# Source-backed literate views, editable comparisons, and isolated visual bisect.
test-literate-views:
    cargo build -p hickory-cli --example engine_client --example acp_fixture
    cargo test -p hickory-cli --test literate_views -- --test-threads=1
    cargo test -p hickory-cli --test serve_acp acp_organizes_and_edits_a_disposable_view -- --test-threads=1
    cargo test -p hickory-cli --test engine_lifecycle -- --test-threads=1
    cd apps/web && npm test -- src/editor/comparison.test.ts src/lsp/representationMapping.test.ts src/lsp/cmLspFeatures.test.ts src/lsp/completion.test.ts


# Shared debugger rails, captured navigation, generated-source routing and keys.
test-unified-code-tools:
    cd apps/web && npm run typecheck && npm test -- --maxWorkers=2 src/debug/DebugRail.test.tsx src/debug/DebugStrip.test.tsx src/debug/plainDebugHosts.test.tsx src/debug/useDebugger.test.ts src/debug/cmDebug.test.ts src/components/ComparisonTools.test.tsx src/lib/keymap.test.ts

test-source-breakpoints:
    cargo test -p hick-dap --lib --test project
    cargo test -p hickory-cli --lib serve::debug_bridge:: -- --test-threads=1
    cargo test -p hickory-cli --test debug_desktop source_breakpoints -- --test-threads=1
    just test-unified-code-tools


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
    cd apps/web && npm run typecheck && npm test -- src/views/ConversationEditor.test.tsx src/components/ChatDock.test.tsx src/editor/session.test.tsx

# Durable model-failure evidence, assumptions, review and historical receipts.
test-agent-audit:
    cargo test -p hickory-cli --lib serve::audit_evidence:: -- --test-threads=1
    cargo test -p hickory-agent --lib --test tools_edit_session -- --test-threads=1
    cargo test -p hickory-cli --test transform_tests --test session_lens --test serve_agent -- --test-threads=1
    cd apps/web && npm run typecheck && npm test -- --maxWorkers=2 src/elements/session/evidence.test.tsx src/components/AssumptionAudit.test.tsx src/components/AgentChanges.test.tsx src/views/ConversationEditor.test.tsx src/components/ChatDock.test.tsx


# Full agent implementation, including historical external tool surfaces.
test-agent-implementation:
    cargo test -p hickory-agent -p hickory-cli --lib -- --test-threads=1
    cargo test -p hickory-cli --test agent_cell_vertex --test session_lens --test transform_tests -- --test-threads=1

# Exercise the locked Python worker and real OS confinement without a model key.
test-python-agent:
    scripts/check-python-agent.sh


# Validate the local guarantee-maintainer skill metadata.
validate-guarantee-skill SKILL_CREATOR_DIR:
    uv run --with pyyaml python "{{SKILL_CREATOR_DIR}}/scripts/quick_validate.py" .codex/skills/guarantee-maintainer

# File menu, independent unnamed buffers, and opening selected files in place.
test-file-menu:
    cd apps/web && npm run typecheck && npm test -- src/router.test.ts
    cargo test -p hickory-cli --test open_selected_file -- --test-threads=1


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
    cd apps/web && npm test -- src/lib/projectComparison.test.ts src/components/ComparisonEditor.test.tsx src/components/ComparisonTools.test.tsx src/debug/documentDebugContext.test.ts src/elements/patch/Editor.test.tsx --maxWorkers=2

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
    cd apps/web && npm test -- --maxWorkers=2

# Compact editor chrome, preferences and native browser geometry.
test-editor-density:
    cd apps/web && npm run typecheck && npm test -- src/shell/PaneHeader.test.tsx src/components/PlainFilePane.test.tsx src/editor/EditorRuler.test.tsx src/lib/keymap.test.ts src/lib/cardRail.test.ts --maxWorkers=2


# Files text rows, floating preview surfaces and themed hints.
test-files-previews:
    cd apps/web && npm run typecheck && npm test -- src/shell/SpatialController.test.ts src/shell/spatialDrawers.test.ts src/lib/tooltip.test.ts --maxWorkers=2


# Install the frontend dependencies from the checked-in lockfile.
setup-web:
    cd apps/web && npm ci

# Folder-scoped open editors, layout restoration and close flushing.
test-folder-window-web:
    just check-web
    cd apps/web && npm test -- --maxWorkers=2 src/shell/spatialLayout.test.ts src/lib/uiState.test.ts


# Literate debugger navigation and exact source-position mapping.
test-debug-navigation:
    cd apps/web && npm test -- src/debug/literateDebugNavigation.test.tsx src/lib/capturedContexts.test.ts src/debug/projectFrameOffset.test.ts src/debug/projectDebugSurface.test.tsx src/debug/DebugRail.test.tsx src/debug/preferredDebugLocation.test.ts src/debug/plainDebugHosts.test.tsx src/components/ComparisonTools.test.tsx src/lsp/representationMapping.test.ts --maxWorkers=2

# File tabs and Files rows share context-menu actions and prompts.
test-file-tab-menu:
    just check-web
    cd apps/web && npm test -- src/shell/FolderTreePane.test.tsx --maxWorkers=2

# Exact-source project selection, breakpoint rails and frozen comparison tools.
test-exact-source-debug:
    cargo test -p hick-lang --test project_debug --test project
    cargo test -p hick-lsp --test project
    cd apps/web && npm run typecheck && npm test -- --maxWorkers=2 src/debug/documentDebugContext.test.ts src/debug/DebugRail.test.tsx src/debug/useDebugger.test.ts src/components/ComparisonTools.test.tsx


# Launch discovery/configuration, atomic project versions and profile chooser.
test-launch-profiles:
    cargo test -p hick-lang --test project_debug --test project
    cargo test -p hickory-cli --test launch_profiles
    cargo test -p hickory-cli --lib serve::test_run
    cd apps/web && npm run typecheck && npm test -- --maxWorkers=2 src/debug/launchProfiles.test.tsx src/debug/projectDebugSurface.test.tsx src/debug/useDebugger.test.ts src/debug/plainDebugHosts.test.tsx src/debug/DebugRail.test.tsx src/components/ComparisonTools.test.tsx


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
    cd apps/web && npm run typecheck && npm test -- --maxWorkers=2 src/components/ChangesetDocument.test.tsx src/elements/patch/Editor.test.tsx src/elements/patch/view.test.tsx src/components/ComparisonTools.test.tsx src/components/AgentChanges.test.tsx src/views/ConversationEditor.test.tsx src/components/ChatDock.test.tsx

check-agent-changesets:
    cargo fmt --all -- --check
    just check-file-length
    cargo clippy -p hickory-cli --lib --example acp_fixture --test agent_changesets -- -D warnings

# Provenance visualization grouping and overlay geometry.
test-ribbon-grouping:
    cd apps/web && npm test -- src/lib/coalesceRibbons.test.ts src/shell/coalesceLinks.test.ts src/shell/linkShapes.test.ts src/lib/ribbonGeometry.test.ts


# Personal agent response recovery and the actual durable store.
test-response-recovery:
    just check-web
    cd apps/web && npm test -- --maxWorkers=2 src/lib/responseRecovery.test.ts src/components/ChatDock.test.tsx src/views/ConversationEditor.test.tsx
    cargo test -p hickory-workspace --lib
    cargo check -p hickory-cli


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

# Prepare the literate toolchain and Git gates, then adopt an exact checkout.
literate-bootstrap: _literate-engine
    cargo build --locked -p hickory-cli --bin hick
    git config --local core.hooksPath .githooks
    ./target/debug/hick project install-hook project.md
    python3 hickory-dogfood/tools/behavior.py sync

# After pulling, adopt matching source/output bytes without generating files.
# Distinct from generation: mismatches refuse without rewriting any product.
literate-sync: _literate-engine
    python3 hickory-dogfood/tools/behavior.py sync

# Rebuild after pulls so synchronization never uses an obsolete adapter.
_literate-engine:
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

# Platform-specific builds for the Simple frontends.
simple-build platform="browser":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ {{quote(platform)}} = windows ]; then
      exec bash scripts/build-simple-windows.sh
    fi
    if [ {{quote(platform)}} = linux ]; then
      if [ "$(uname -s)" != Linux ]; then
        echo 'Run just simple-build linux on Linux with CMake, Qt 6.8 WebEngine Widgets and KDE Frameworks 6 development packages.' >&2
        exit 2
      fi
      build="$PWD/target/simple-linux"
      if [ ! -x apps/web/node_modules/.bin/vite ] || [ apps/web/package-lock.json -nt apps/web/node_modules ]; then
        npm --prefix apps/web ci
      fi
      # Build the same entry point with compiled assets beside the native host.
      (cd apps/web && node node_modules/vite/bin/vite.js build --config vite.simple.config.ts --outDir "$build/web" --emptyOutDir)
      cargo build --locked --target-dir "$PWD/target" -p hickory-cli --example simple_dev
      cmake -S apps/linux-simple -B "$build/cmake" -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_INSTALL_BINDIR=bin -DHICKORY_SIMPLE_HOST="$PWD/target/debug/examples/simple_dev" \
        -DHICKORY_SIMPLE_WEB="$build/web"
      cmake --build "$build/cmake"
      cmake --install "$build/cmake" --prefix "$build/app"
      exit
    fi
    case {{quote(platform)}} in
      apple|ios) DEVELOPER_DIR="$(just _xcode-developer-dir)"; export DEVELOPER_DIR ;;
    esac
    python3 scripts/prepare-simple-resources.py
    case {{quote(platform)}} in
      browser) cd apps/web; npx vite build --config vite.simple.config.ts ;;
      apple) swift build --package-path apps/apple-simple --scratch-path target/apple-simple-spm ;;
      ios) xcodegen generate --spec apps/apple-simple/project.yml; xcodebuild -project apps/apple-simple/HickorySimple.xcodeproj -scheme HickorySimpleMobile -sdk iphonesimulator -destination 'generic/platform=iOS Simulator' -derivedDataPath target/apple-simple-derived CODE_SIGNING_ALLOWED=NO build ;;
      android) mkdir -p target; rustc --edition=2021 scripts/simple-android-gradle.rs -o target/simple-android-gradle; target/simple-android-gradle :app:assembleDebug ;;
      *) echo 'Choose browser, apple, ios, android, linux or windows.' >&2; exit 2 ;;
    esac

# Focused parity/safety witnesses only, with platform toolchain prerequisites.
simple-test platform="browser":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ {{quote(platform)}} = windows ]; then
      just simple-build windows
      node scripts/test-simple-windows.mjs
      exit
    fi
    if [ {{quote(platform)}} = linux ]; then
      just simple-build linux
      cargo test --locked -p hickory-cli --example simple_dev linux_host_admits_only_its_own_origin
      node scripts/test-simple-linux.mjs
      exit
    fi
    if [ {{quote(platform)}} = apple ]; then
      DEVELOPER_DIR="$(just _xcode-developer-dir)"
      export DEVELOPER_DIR
    fi
    python3 scripts/prepare-simple-resources.py
    case {{quote(platform)}} in
      browser) cd apps/web; npx tsc -b --noEmit; npx vitest run src/simple/parity.test.ts; npx vite build --config vite.simple.config.ts ;;
      apple) swift test --package-path apps/apple-simple --scratch-path target/apple-simple-spm; swiftc scripts/simple-window-layout.swift -o target/simple-window-layout; target/simple-window-layout --self-test ;;
      android) mkdir -p target; rustc --edition=2021 scripts/simple-android-gradle.rs -o target/simple-android-gradle; target/simple-android-gradle :app:testDebugUnitTest :app:assembleDebug ;;
      *) echo 'Choose browser, apple, android, linux or windows.' >&2; exit 2 ;;
    esac

# Four real engine/debugpy/native-controller flows; prepares locked local dependencies.
native-debug-smoke platform:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p target
    if [ ! -x target/native-debug-smoke-runner ] || [ scripts/native-debug-smoke.rs -nt target/native-debug-smoke-runner ]; then rustc --edition 2024 scripts/native-debug-smoke.rs -o target/native-debug-smoke-runner; fi
    target/native-debug-smoke-runner {{quote(platform)}}

# Commits rebuild and relaunch the selected frontend.
[env("HICKORY_DEV_BUILD", "1")]
simple-dev platform="browser": _dev-supervisor
    {{ if platform == "browser" { "target/dev-supervisor --desktop " } else if platform == "apple" { "target/dev-supervisor --desktop " } else if platform == "linux" { "target/dev-supervisor --desktop " } else if platform == "windows" { "target/dev-supervisor.exe --desktop " } else { "" } }}cargo run {{ if platform == "apple" { "" } else { "--quiet " } }}--locked -p hickory-cli --example simple_dev -- {{quote(platform)}}

# Install the user-level Codex Stop hook for this repository on macOS.
codex-simple-dev-install: _dev-supervisor
    python3 scripts/codex-simple-dev.py install

# Real native update/relaunch and app-data preservation on an explicitly selected device.
simple-reload-test platform:
    python3 scripts/test-simple-mobile-reload.py {{quote(platform)}}

# Compile the dependency-free Rust lifecycle tool; no Cargo workspace build needed.
_dev-supervisor:
    @mkdir -p target
    @if [ ! -x target/dev-supervisor ] || [ scripts/dev-supervisor.rs -nt target/dev-supervisor ]; then rustc --edition 2024 scripts/dev-supervisor.rs -o target/dev-supervisor; fi
</maint:entry>
</maint:doc>
