//! WASM container lifecycle management.
//!
//! This crate provides the core abstractions for running WebAssembly modules
//! as isolated containers with programmatic I/O, optional networking, and
//! cooperative pause/resume support. Containers execute in background tokio
//! tasks and communicate with the host through Unix pipe pairs bridged to
//! WASI.

pub mod container;
pub mod pipe;

/// A running WASM container with programmatic stdin/stdout/stderr I/O.
pub use container::Container;
/// Configuration for starting a [`Container`].
pub use container::ContainerConfig;
/// Snapshot of container resource usage.
pub use container::ContainerStats;
/// Create a [`wasmtime::Engine`] configured for async operation with epoch interruption.
pub use container::create_async_engine;
