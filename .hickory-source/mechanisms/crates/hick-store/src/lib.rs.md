<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-store/src/lib.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-93f949dfdf41"></a>
## `crates/hick-store/src/lib.rs`

<a id="f-93f949dfdf41-builtin_backend"></a>
### `builtin_backend`

Versioned pipeline state storage.
Provides a `VersionStore` trait for storing snapshots, blobs, and branches,
with implementations backed by git plumbing commands and a built-in
filesystem/in-memory store.

<storycode:copy id="f-93f949dfdf41-builtin_backend">
//! Versioned pipeline state storage.
//!
//! Provides a `VersionStore` trait for storing snapshots, blobs, and branches,
//! with implementations backed by git plumbing commands and a built-in
//! filesystem/in-memory store.

pub mod builtin_backend;
</storycode:copy>
<a id="f-93f949dfdf41-error"></a>
### `error`

<storycode:copy id="f-93f949dfdf41-error">
pub mod error;
</storycode:copy>
<a id="f-93f949dfdf41-fs_object_store"></a>
### `fs_object_store`

<storycode:copy id="f-93f949dfdf41-fs_object_store">
pub mod fs_object_store;
</storycode:copy>
<a id="f-93f949dfdf41-git_backend"></a>
### `git_backend`

<storycode:copy id="f-93f949dfdf41-git_backend">
pub mod git_backend;
</storycode:copy>
<a id="f-93f949dfdf41-memory_store"></a>
### `memory_store`

<storycode:copy id="f-93f949dfdf41-memory_store">
pub mod memory_store;
</storycode:copy>
<a id="f-93f949dfdf41-object_store"></a>
### `object_store`

<storycode:copy id="f-93f949dfdf41-object_store">
pub mod object_store;
</storycode:copy>
<a id="f-93f949dfdf41-s3_object_store"></a>
### `s3_object_store`

<storycode:copy id="f-93f949dfdf41-s3_object_store">
#[cfg(feature = "s3")]
pub mod s3_object_store;
</storycode:copy>
<a id="f-93f949dfdf41-types"></a>
### `types`

<storycode:copy id="f-93f949dfdf41-types">
pub mod types;
</storycode:copy>
<a id="f-93f949dfdf41-version_store"></a>
### `version_store`

<storycode:copy id="f-93f949dfdf41-version_store">
pub mod version_store;

</storycode:copy>
<a id="f-93f949dfdf41-use_declaration"></a>
### `use_declaration`

[`VersionStore`] backed by any [`ObjectStore`], using content-addressed layout.

<storycode:copy id="f-93f949dfdf41-use_declaration">
/// [`VersionStore`] backed by any [`ObjectStore`], using content-addressed layout.
pub use builtin_backend::BuiltinVersionStore;
</storycode:copy>
<a id="f-93f949dfdf41-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-93f949dfdf41-use_declaration-2">
/// Error type for version store operations.
pub use error::StoreError;
</storycode:copy>
<a id="f-93f949dfdf41-use_declaration-3"></a>
### `use_declaration`

Filesystem-backed [`ObjectStore`] that maps keys to files under a root directory.

<storycode:copy id="f-93f949dfdf41-use_declaration-3">
/// Filesystem-backed [`ObjectStore`] that maps keys to files under a root directory.
pub use fs_object_store::FsObjectStore;
</storycode:copy>
<a id="f-93f949dfdf41-use_declaration-4"></a>
### `use_declaration`

[`VersionStore`] backed by a bare git repository using plumbing commands.

<storycode:copy id="f-93f949dfdf41-use_declaration-4">
/// [`VersionStore`] backed by a bare git repository using plumbing commands.
pub use git_backend::GitVersionStore;
</storycode:copy>
<a id="f-93f949dfdf41-use_declaration-5"></a>
### `use_declaration`

In-memory [`ObjectStore`] for testing and as a reference implementation.

<storycode:copy id="f-93f949dfdf41-use_declaration-5">
/// In-memory [`ObjectStore`] for testing and as a reference implementation.
pub use memory_store::InMemoryObjectStore;
</storycode:copy>
<a id="f-93f949dfdf41-use_declaration-6"></a>
### `use_declaration`

Low-level key-value storage trait used by [`BuiltinVersionStore`].

<storycode:copy id="f-93f949dfdf41-use_declaration-6">
/// Low-level key-value storage trait used by [`BuiltinVersionStore`].
pub use object_store::ObjectStore;
</storycode:copy>
<a id="f-93f949dfdf41-use_declaration-7"></a>
### `use_declaration`

[`ObjectStore`] backed by Amazon S3 with configurable bucket and prefix.

<storycode:copy id="f-93f949dfdf41-use_declaration-7">
/// [`ObjectStore`] backed by Amazon S3 with configurable bucket and prefix.
#[cfg(feature = "s3")]
pub use s3_object_store::S3ObjectStore;
</storycode:copy>
<a id="f-93f949dfdf41-use_declaration-8"></a>
### `use_declaration`

Content-addressed blob hash, file provenance metadata, immutable snapshot, and snapshot identifier.

<storycode:copy id="f-93f949dfdf41-use_declaration-8">
/// Content-addressed blob hash, file provenance metadata, immutable snapshot, and snapshot identifier.
pub use types::{BlobHash, FileProvenance, Snapshot, SnapshotId};
</storycode:copy>
<a id="f-93f949dfdf41-use_declaration-9"></a>
### `use_declaration`

Core trait for storing and retrieving versioned pipeline state.

<storycode:copy id="f-93f949dfdf41-use_declaration-9">
/// Core trait for storing and retrieving versioned pipeline state.
pub use version_store::VersionStore;
</storycode:copy>

### Assemble `crates/hick-store/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-store/src/lib.rs"><storycode:paste select="#f-93f949dfdf41-builtin_backend"/><storycode:paste select="#f-93f949dfdf41-error"/><storycode:paste select="#f-93f949dfdf41-fs_object_store"/><storycode:paste select="#f-93f949dfdf41-git_backend"/><storycode:paste select="#f-93f949dfdf41-memory_store"/><storycode:paste select="#f-93f949dfdf41-object_store"/><storycode:paste select="#f-93f949dfdf41-s3_object_store"/><storycode:paste select="#f-93f949dfdf41-types"/><storycode:paste select="#f-93f949dfdf41-version_store"/><storycode:paste select="#f-93f949dfdf41-use_declaration"/><storycode:paste select="#f-93f949dfdf41-use_declaration-2"/><storycode:paste select="#f-93f949dfdf41-use_declaration-3"/><storycode:paste select="#f-93f949dfdf41-use_declaration-4"/><storycode:paste select="#f-93f949dfdf41-use_declaration-5"/><storycode:paste select="#f-93f949dfdf41-use_declaration-6"/><storycode:paste select="#f-93f949dfdf41-use_declaration-7"/><storycode:paste select="#f-93f949dfdf41-use_declaration-8"/><storycode:paste select="#f-93f949dfdf41-use_declaration-9"/></storycode:file>

</storycode:doc>
