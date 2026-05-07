# Pre-commit

Before committing, run the full test suite:

```bash
cargo test --workspace
```

# Semantic Versioning

This repository follows [Semantic Versioning](https://semver.org/).

## Public API

The **public API** is defined by the `pub use` re-exports in each crate's `lib.rs`. Items accessible only through `pub mod` sub-module paths are internal implementation details and may change without notice.

When modifying code:
- Default to `pub(crate)` for new internal helpers.
- Only add items to the `pub use` facade when they are part of the intended contract.
- Never use `pub use module::*` — be explicit so the API surface is auditable.

## Version Bumps

While pre-1.0 (`0.x.y`):
- **Patch** (`0.1.x`): bug fixes, internal refactors, documentation, performance improvements.
- **Minor** (`0.x.0`): new public API items, or breaking changes to existing public API.

After 1.0:
- **Patch**: backwards-compatible bug fixes.
- **Minor**: backwards-compatible new functionality.
- **Major**: breaking changes.

## Backwards Compatibility

1. Never remove or rename a `pub use`'d item without a version bump.
2. Never change a public function/method signature in a breaking way without a version bump.
3. Adding new `pub use` items is always safe (patch-level).
4. Changing anything behind `pub(crate)` or private is always safe — this is freedom to refactor.

## Documentation

- All `pub use`'d items must have `///` doc comments.
- Update documentation when changing public API.
- The `lib.rs` module doc (`//!`) should describe the crate's purpose and key types.
