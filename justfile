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

# Run the server + web dev environment.
dev:
    ./scripts/dev.sh

# Stop anything `just dev` started.
dev-stop:
    ./scripts/dev.sh stop

# Run a hick document with the local executor.
run DOC *ARGS:
    cargo run -p hickory-cli -- run {{DOC}} {{ARGS}}

# Verify documents (drift/expectations). CI + pre-commit entry point.
verify DOC *ARGS:
    cargo run -p hickory-cli -- check {{DOC}} {{ARGS}}

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
