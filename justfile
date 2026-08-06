# Hickory Docs task runner — the only entry point for project tasks.

# Build the whole Rust workspace.
build:
    cargo build --workspace

# Type-check quickly.
check:
    cargo check --workspace

# Run all tests.
test:
    cargo test --workspace

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
