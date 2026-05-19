# c2w-testbed

Thin CLI binary that exposes the `c2w` library for manual testing.

Parses `c2w::cli::Cli` via `clap` and calls `c2w::run`. Exists as a separate
binary crate to keep the testbed build-time footprint independent of the
library; the library crate does not expose a `main`.
