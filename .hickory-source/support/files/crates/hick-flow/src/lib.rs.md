<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-flow/src/lib.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `ba6c5d03bfba4cc7d5ff5d768d2d72eae753c10385f9154bcaeeadc85bfe9613`.

<book:entry path="crates/hick-flow/src/lib.rs">
//! Core reactive data-flow primitives for hickory-docs.
//!
//! This crate provides the fundamental building blocks — `Node`, `NodeTrace`,
//! `InsertionPoint`, `DynamicCombineLatest`, and `TransformNode` — that form
//! the async stream tree powering document generation.

pub mod combine_latest;
pub mod context;
pub mod converge;
pub mod insertion_point;
pub mod node;
pub mod provenance;
pub mod transform;

pub use combine_latest::DynamicCombineLatest;
pub use context::Context;
pub use converge::{converge, converge_to_string, converge_with_provenance};
pub use insertion_point::InsertionPoint;
pub use node::{
    BinaryData, BinaryNode, BoxStream, FileContent, Node, NodeTrace, NodeValue, SeparatorNode,
    SourceOrigin, SpanNode, StringNode,
};
pub use provenance::{ProvenanceMap, ProvenanceSpan};
pub use transform::{
    ProvenanceTransformNode, TransformNode, TransformSegment, TransformSegmentOrigin,
};
</book:entry>

</book:doc>
