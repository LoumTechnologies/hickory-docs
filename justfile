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
    cargo test --workspace -- --test-threads=1

# Lint (warnings are errors, matching CI).
clippy:
    cargo clippy --workspace -- -D warnings

fmt:
    cargo fmt --all

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

