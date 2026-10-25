<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-store/src/lib.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `ebb0237fd57ba15e1cf6ab2938585cc44882088a2e7c5721f1707eedb578fac1`.

<book:entry path="crates/hick-store/src/lib.rs">
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
</book:entry>

</book:doc>
