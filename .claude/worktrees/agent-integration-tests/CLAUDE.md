# Pre-commit

Before committing, always run the full test suite:

```bash
cargo test --workspace -- --test-threads=1
```

# Crucible

When working on Crucible (in `crucible/`), read and follow [crucible/AGENTS.md](crucible/AGENTS.md). Hickory docs integration is a core requirement.
