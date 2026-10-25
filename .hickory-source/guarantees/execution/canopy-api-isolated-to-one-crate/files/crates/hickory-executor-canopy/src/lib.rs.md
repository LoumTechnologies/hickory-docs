<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-executor-canopy/src/lib.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Only `hickory-executor-canopy` Knows Cloud Canopy's API](../../../../contract.md)

Captured SHA-256: `3dad7de3d41f86fe8417a05e8b4ab986c32e532dd8876be415aa9536def10b62`.

<book:entry path="crates/hickory-executor-canopy/src/lib.rs">
//! Cloud Canopy executor for Hickory Docs.
//!
//! This crate is the ONLY code in the repository that knows Cloud Canopy's
//! API (see `docs/guarantees/execution/canopy-api-isolated-to-one-crate.md`).
//! The entire coupling is the vendored `proto/canopy.proto` — never add a
//! path or git dependency on a cloud-canopy crate.
//!
//! See [`CanopyExecutor`] for runtime semantics and
//! `docs/specs/freeform/canopy-integration.md` for the env contract.

mod config;
mod executor;
mod frame;

/// Generated gRPC bindings for the vendored `proto/canopy.proto`.
///
/// Public so the in-crate contract tests can implement a mock
/// `CanopyAgent` server against the exact wire contract. Nothing outside
/// this crate may use it (enforced by `tests/isolation.rs`).
#[allow(clippy::all, clippy::pedantic)]
pub mod pb {
    tonic::include_proto!("canopy.v1");
}

pub use config::{CanopyConfig, image_map_summary};
pub use executor::CanopyExecutor;

// Exposed for the in-crate contract tests' mock guest; not a public API.
#[doc(hidden)]
pub use frame::{
    FrameItem, FrameParser, READY_BANNER, SENTINEL_END, SENTINEL_START, decode_script,
    encode_script, exec_line, shell_quote, strip_ansi,
};
</book:entry>

</book:doc>
