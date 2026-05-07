//! Versioned pipeline state storage.
//!
//! Provides a `VersionStore` trait for storing snapshots, blobs, and branches,
//! with implementations backed by git plumbing commands and a built-in
//! filesystem/in-memory store.

pub mod builtin_backend;
pub mod error;
pub mod fs_object_store;
pub mod git_backend;
pub mod memory_store;
pub mod object_store;
#[cfg(feature = "s3")]
pub mod s3_object_store;
pub mod types;
pub mod version_store;

/// [`VersionStore`] backed by any [`ObjectStore`], using content-addressed layout.
pub use builtin_backend::BuiltinVersionStore;
/// Error type for version store operations.
pub use error::StoreError;
/// Filesystem-backed [`ObjectStore`] that maps keys to files under a root directory.
pub use fs_object_store::FsObjectStore;
/// [`VersionStore`] backed by a bare git repository using plumbing commands.
pub use git_backend::GitVersionStore;
/// In-memory [`ObjectStore`] for testing and as a reference implementation.
pub use memory_store::InMemoryObjectStore;
/// Low-level key-value storage trait used by [`BuiltinVersionStore`].
pub use object_store::ObjectStore;
/// [`ObjectStore`] backed by Amazon S3 with configurable bucket and prefix.
#[cfg(feature = "s3")]
pub use s3_object_store::S3ObjectStore;
/// Content-addressed blob hash, file provenance metadata, immutable snapshot, and snapshot identifier.
pub use types::{BlobHash, FileProvenance, Snapshot, SnapshotId};
/// Core trait for storing and retrieving versioned pipeline state.
pub use version_store::VersionStore;
