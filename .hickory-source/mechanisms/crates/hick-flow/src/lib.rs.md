<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-flow/src/lib.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-084eb2e8cf67"></a>
## `crates/hick-flow/src/lib.rs`

<a id="f-084eb2e8cf67-combine_latest"></a>
### `combine_latest`

Core reactive data-flow primitives for hickory-docs.
This crate provides the fundamental building blocks — `Node`, `NodeTrace`,
`InsertionPoint`, `DynamicCombineLatest`, and `TransformNode` — that form
the async stream tree powering document generation.

<storycode:copy id="f-084eb2e8cf67-combine_latest">
//! Core reactive data-flow primitives for hickory-docs.
//!
//! This crate provides the fundamental building blocks — `Node`, `NodeTrace`,
//! `InsertionPoint`, `DynamicCombineLatest`, and `TransformNode` — that form
//! the async stream tree powering document generation.

pub mod combine_latest;
</storycode:copy>
<a id="f-084eb2e8cf67-context"></a>
### `context`

<storycode:copy id="f-084eb2e8cf67-context">
pub mod context;
</storycode:copy>
<a id="f-084eb2e8cf67-converge"></a>
### `converge`

<storycode:copy id="f-084eb2e8cf67-converge">
pub mod converge;
</storycode:copy>
<a id="f-084eb2e8cf67-insertion_point"></a>
### `insertion_point`

<storycode:copy id="f-084eb2e8cf67-insertion_point">
pub mod insertion_point;
</storycode:copy>
<a id="f-084eb2e8cf67-node"></a>
### `node`

<storycode:copy id="f-084eb2e8cf67-node">
pub mod node;
</storycode:copy>
<a id="f-084eb2e8cf67-provenance"></a>
### `provenance`

<storycode:copy id="f-084eb2e8cf67-provenance">
pub mod provenance;
</storycode:copy>
<a id="f-084eb2e8cf67-transform"></a>
### `transform`

<storycode:copy id="f-084eb2e8cf67-transform">
pub mod transform;

</storycode:copy>
<a id="f-084eb2e8cf67-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-084eb2e8cf67-use_declaration">
pub use combine_latest::DynamicCombineLatest;
</storycode:copy>
<a id="f-084eb2e8cf67-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-084eb2e8cf67-use_declaration-2">
pub use context::Context;
</storycode:copy>
<a id="f-084eb2e8cf67-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-084eb2e8cf67-use_declaration-3">
pub use converge::{converge, converge_to_string, converge_with_provenance};
</storycode:copy>
<a id="f-084eb2e8cf67-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-084eb2e8cf67-use_declaration-4">
pub use insertion_point::InsertionPoint;
</storycode:copy>
<a id="f-084eb2e8cf67-use_declaration-5"></a>
### `use_declaration`

<storycode:copy id="f-084eb2e8cf67-use_declaration-5">
pub use node::{
    BinaryData, BinaryNode, BoxStream, FileContent, Node, NodeTrace, NodeValue, SeparatorNode,
    SourceOrigin, SpanNode, StringNode,
};
</storycode:copy>
<a id="f-084eb2e8cf67-use_declaration-6"></a>
### `use_declaration`

<storycode:copy id="f-084eb2e8cf67-use_declaration-6">
pub use provenance::{ProvenanceMap, ProvenanceSpan};
</storycode:copy>
<a id="f-084eb2e8cf67-use_declaration-7"></a>
### `use_declaration`

<storycode:copy id="f-084eb2e8cf67-use_declaration-7">
pub use transform::{
    ProvenanceTransformNode, TransformNode, TransformSegment, TransformSegmentOrigin,
};
</storycode:copy>

### Assemble `crates/hick-flow/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-flow/src/lib.rs"><storycode:paste select="#f-084eb2e8cf67-combine_latest"/><storycode:paste select="#f-084eb2e8cf67-context"/><storycode:paste select="#f-084eb2e8cf67-converge"/><storycode:paste select="#f-084eb2e8cf67-insertion_point"/><storycode:paste select="#f-084eb2e8cf67-node"/><storycode:paste select="#f-084eb2e8cf67-provenance"/><storycode:paste select="#f-084eb2e8cf67-transform"/><storycode:paste select="#f-084eb2e8cf67-use_declaration"/><storycode:paste select="#f-084eb2e8cf67-use_declaration-2"/><storycode:paste select="#f-084eb2e8cf67-use_declaration-3"/><storycode:paste select="#f-084eb2e8cf67-use_declaration-4"/><storycode:paste select="#f-084eb2e8cf67-use_declaration-5"/><storycode:paste select="#f-084eb2e8cf67-use_declaration-6"/><storycode:paste select="#f-084eb2e8cf67-use_declaration-7"/></storycode:file>

</storycode:doc>
