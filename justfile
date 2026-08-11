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
    VITE_POSTHOG_KEY="$key" npm run build
    echo
    echo "Static site built: apps/web/dist"
    echo "It needs no backend. The landing page and its demos run in the browser."
    echo "Serves /install.sh — note that the installer pulls from GitHub releases,"
    echo "which strangers can only reach once the repository is public."

# Mint a KEY_ENCRYPTION_KEY (encrypts accounts' own provider API keys).
# One per environment; replacing it invalidates every stored key.
gen-key:
    @cargo run -q -p hickory-server --bin gen-key

# Run the server + web dev environment.
dev:
    ./scripts/dev.sh

# Stop anything `just dev` started.
dev-stop:
    ./scripts/dev.sh stop

# Stop, then delete this worktree's containers/volumes (never another
# worktree's, never a pulled base image).
dev-clean:
    ./scripts/dev.sh clean

# Idempotent seed data via the real signup endpoint (never a DB insert).
dev-seed:
    ./scripts/dev-seed.sh

# Regenerate the OpenAPI spec + typed web client. No infra required.
codegen:
    ./scripts/codegen.sh

# codegen + fail if the committed output is stale. CI + pre-commit entry point.
check-codegen:
    ./scripts/check-codegen.sh

# Build one downloadable artifact (binary + LICENSE + examples) into dist/.
# The same script the release workflows call, so a maintainer can reproduce
# what CI ships. Targets:
#   x86_64-unknown-linux-musl  aarch64-unknown-linux-musl
#   aarch64-apple-darwin       x86_64-apple-darwin
#   x86_64-pc-windows-msvc
dist TARGET VERSION="":
    ./scripts/dist.sh {{TARGET}} {{VERSION}}

# Run a hick document with the local executor.
run DOC *ARGS:
    cargo run -p hickory-cli -- run {{DOC}} {{ARGS}}

# Verify documents (drift/expectations). CI + pre-commit entry point.
verify DOC *ARGS:
    cargo run -p hickory-cli -- test {{DOC}} {{ARGS}}

# Run the agent on a prompt. Loads .env first, which is where API keys live —
# the CLI itself does not read .env, so calling `hickory agent` directly needs
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

# Terraform creates the analytics project and knows its write key; the server
# needs that key as POSTHOG_API_KEY. No provider bridges the two, so this is
# the explicit hand-off (the documented fallback in
# .instructions/continuous-delivery-paas.md). Run it after applying the
# posthog stack. Details: docs/operators/analytics.md.

# Push the Terraform-owned PostHog project key to the Fly app.
posthog-sync-key:
    ./scripts/posthog-sync-key.sh
