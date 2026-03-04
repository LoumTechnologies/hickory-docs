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

pub use builtin_backend::BuiltinVersionStore;
pub use error::StoreError;
pub use fs_object_store::FsObjectStore;
pub use git_backend::GitVersionStore;
pub use memory_store::InMemoryObjectStore;
pub use object_store::ObjectStore;
#[cfg(feature = "s3")]
pub use s3_object_store::S3ObjectStore;
pub use types::{BlobHash, FileProvenance, Snapshot, SnapshotId};
pub use version_store::VersionStore;
