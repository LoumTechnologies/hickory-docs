<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-store/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `4eb20b4847968244c7b2599f5cc29d9cc4f264cdd3aac8187d96c7886d117928`.

<book:entry path="crates/hick-store/README.md">
# hick-store

Versioned pipeline state storage.

The `VersionStore` trait provides an interface for storing and retrieving
immutable snapshots, content-addressed blobs (`BlobHash`), and named branches.

## Implementations

| Backend | Description |
|---|---|
| `GitVersionStore` | Backed by git plumbing on a bare repository |
| `BuiltinVersionStore` | Content-addressed layout over any `ObjectStore` |
| `FsObjectStore` | Filesystem key-value store |
| `InMemoryObjectStore` | In-memory reference implementation |
| `S3ObjectStore` | S3-backed object store (optional) |

## Key types

- `Snapshot` / `SnapshotId` — immutable pipeline output snapshot
- `FileProvenance` — tracks which pipeline run produced each file

Used by the merge orchestrator in `hick-merge` to find the base version for
three-way merges when user edits need to be reconciled with new pipeline output.
</book:entry>

</book:doc>
