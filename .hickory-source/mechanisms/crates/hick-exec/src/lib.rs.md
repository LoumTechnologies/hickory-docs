<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-exec/src/lib.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-2f730ff897ea"></a>
## `crates/hick-exec/src/lib.rs`

<a id="f-2f730ff897ea-combine_latest"></a>
### `combine_latest`

Execution engine for `.md` files.
Ports `DynamicCombineLatest`, `Node`, `InsertionPoint`, and related types from
HickoryDocs, and adds information-flow DAG construction and validation.

<storycode:copy id="f-2f730ff897ea-combine_latest">
//! Execution engine for `.md` files.
//!
//! Ports `DynamicCombineLatest`, `Node`, `InsertionPoint`, and related types from
//! HickoryDocs, and adds information-flow DAG construction and validation.

pub mod combine_latest;
</storycode:copy>
<a id="f-2f730ff897ea-dag"></a>
### `dag`

<storycode:copy id="f-2f730ff897ea-dag">
pub mod dag;
</storycode:copy>
<a id="f-2f730ff897ea-node"></a>
### `node`

<storycode:copy id="f-2f730ff897ea-node">
pub mod node;
</storycode:copy>
<a id="f-2f730ff897ea-state"></a>
### `state`

<storycode:copy id="f-2f730ff897ea-state">
pub mod state;
</storycode:copy>
<a id="f-2f730ff897ea-volume"></a>
### `volume`

<storycode:copy id="f-2f730ff897ea-volume">
pub mod volume;

</storycode:copy>
<a id="f-2f730ff897ea-use_declaration"></a>
### `use_declaration`

Combines multiple async streams, emitting the latest merged value whenever any source updates.

<storycode:copy id="f-2f730ff897ea-use_declaration">
/// Combines multiple async streams, emitting the latest merged value whenever any source updates.
pub use combine_latest::DynamicCombineLatest;

</storycode:copy>
<a id="f-2f730ff897ea-use_declaration-2"></a>
### `use_declaration`

What a `<hick:agent>` cell declares: prompt, model, turn budget, identity.

<storycode:copy id="f-2f730ff897ea-use_declaration-2">
/// What a `<hick:agent>` cell declares: prompt, model, turn budget, identity.
pub use dag::AgentCell;

</storycode:copy>
<a id="f-2f730ff897ea-use_declaration-3"></a>
### `use_declaration`

Identity of an agent cell that survives the agent editing its own document.

<storycode:copy id="f-2f730ff897ea-use_declaration-3">
/// Identity of an agent cell that survives the agent editing its own document.
pub use dag::AgentKey;

</storycode:copy>
<a id="f-2f730ff897ea-use_declaration-4"></a>
### `use_declaration`

The reserved synthetic container name of the agent cell at a given index.

<storycode:copy id="f-2f730ff897ea-use_declaration-4">
/// The reserved synthetic container name of the agent cell at a given index.
pub use dag::agent_container_name;

</storycode:copy>
<a id="f-2f730ff897ea-use_declaration-5"></a>
### `use_declaration`

Whether a container name is an agent cell's reserved synthetic name.

<storycode:copy id="f-2f730ff897ea-use_declaration-5">
/// Whether a container name is an agent cell's reserved synthetic name.
pub use dag::is_agent_container;

</storycode:copy>
<a id="f-2f730ff897ea-use_declaration-6"></a>
### `use_declaration`

An edge in the information-flow DAG representing a dependency between exec elements.

<storycode:copy id="f-2f730ff897ea-use_declaration-6">
/// An edge in the information-flow DAG representing a dependency between exec elements.
pub use dag::DagEdge;

</storycode:copy>
<a id="f-2f730ff897ea-use_declaration-7"></a>
### `use_declaration`

Error produced when the information-flow DAG contains cycles or broken references.

<storycode:copy id="f-2f730ff897ea-use_declaration-7">
/// Error produced when the information-flow DAG contains cycles or broken references.
pub use dag::DagValidationError;

</storycode:copy>
<a id="f-2f730ff897ea-use_declaration-8"></a>
### `use_declaration`

Unique identifier for an `<hick:exec>` element, assigned by document order.

<storycode:copy id="f-2f730ff897ea-use_declaration-8">
/// Unique identifier for an `<hick:exec>` element, assigned by document order.
pub use dag::ExecId;

</storycode:copy>
<a id="f-2f730ff897ea-use_declaration-9"></a>
### `use_declaration`

The validated information-flow DAG of exec elements, their edges, and root nodes.

<storycode:copy id="f-2f730ff897ea-use_declaration-9">
/// The validated information-flow DAG of exec elements, their edges, and root nodes.
pub use dag::FlowDag;

</storycode:copy>
<a id="f-2f730ff897ea-use_declaration-10"></a>
### `use_declaration`

Type-erased pinned async `Stream` used throughout the node graph.

<storycode:copy id="f-2f730ff897ea-use_declaration-10">
/// Type-erased pinned async `Stream` used throughout the node graph.
pub use node::BoxStream;

</storycode:copy>
<a id="f-2f730ff897ea-use_declaration-11"></a>
### `use_declaration`

Per-evaluation context passed through the reactive node graph.

<storycode:copy id="f-2f730ff897ea-use_declaration-11">
/// Per-evaluation context passed through the reactive node graph.
pub use node::Context;

</storycode:copy>
<a id="f-2f730ff897ea-use_declaration-12"></a>
### `use_declaration`

Ordered collection of child nodes whose outputs are concatenated in sequence.

<storycode:copy id="f-2f730ff897ea-use_declaration-12">
/// Ordered collection of child nodes whose outputs are concatenated in sequence.
pub use node::InsertionPoint;

</storycode:copy>
<a id="f-2f730ff897ea-use_declaration-13"></a>
### `use_declaration`

Core trait for reactive document nodes that produce streaming output.

<storycode:copy id="f-2f730ff897ea-use_declaration-13">
/// Core trait for reactive document nodes that produce streaming output.
pub use node::Node;

</storycode:copy>
<a id="f-2f730ff897ea-use_declaration-14"></a>
### `use_declaration`

Trace record capturing which nodes contributed to a converged result.

<storycode:copy id="f-2f730ff897ea-use_declaration-14">
/// Trace record capturing which nodes contributed to a converged result.
pub use node::NodeTrace;

</storycode:copy>
<a id="f-2f730ff897ea-use_declaration-15"></a>
### `use_declaration`

Node that emits a fixed separator string between sibling outputs.

<storycode:copy id="f-2f730ff897ea-use_declaration-15">
/// Node that emits a fixed separator string between sibling outputs.
pub use node::SeparatorNode;

</storycode:copy>
<a id="f-2f730ff897ea-use_declaration-16"></a>
### `use_declaration`

Leaf node that emits a single static string value.

<storycode:copy id="f-2f730ff897ea-use_declaration-16">
/// Leaf node that emits a single static string value.
pub use node::StringNode;

</storycode:copy>
<a id="f-2f730ff897ea-use_declaration-17"></a>
### `use_declaration`

Node that applies a mapping function to the output of a child node.

<storycode:copy id="f-2f730ff897ea-use_declaration-17">
/// Node that applies a mapping function to the output of a child node.
pub use node::TransformNode;

</storycode:copy>
<a id="f-2f730ff897ea-use_declaration-18"></a>
### `use_declaration`

Converges a node graph into a single `String`, collecting all streamed segments.

<storycode:copy id="f-2f730ff897ea-use_declaration-18">
/// Converges a node graph into a single `String`, collecting all streamed segments.
pub use node::converge_to_string;

</storycode:copy>
<a id="f-2f730ff897ea-use_declaration-19"></a>
### `use_declaration`

Global mutable state shared across a hick execution run, tracking files, copy/paste blocks, and variables.

<storycode:copy id="f-2f730ff897ea-use_declaration-19">
/// Global mutable state shared across a hick execution run, tracking files, copy/paste blocks, and variables.
pub use state::MultiDocumentState;
</storycode:copy>

### Assemble `crates/hick-exec/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-exec/src/lib.rs"><storycode:paste select="#f-2f730ff897ea-combine_latest"/><storycode:paste select="#f-2f730ff897ea-dag"/><storycode:paste select="#f-2f730ff897ea-node"/><storycode:paste select="#f-2f730ff897ea-state"/><storycode:paste select="#f-2f730ff897ea-volume"/><storycode:paste select="#f-2f730ff897ea-use_declaration"/><storycode:paste select="#f-2f730ff897ea-use_declaration-2"/><storycode:paste select="#f-2f730ff897ea-use_declaration-3"/><storycode:paste select="#f-2f730ff897ea-use_declaration-4"/><storycode:paste select="#f-2f730ff897ea-use_declaration-5"/><storycode:paste select="#f-2f730ff897ea-use_declaration-6"/><storycode:paste select="#f-2f730ff897ea-use_declaration-7"/><storycode:paste select="#f-2f730ff897ea-use_declaration-8"/><storycode:paste select="#f-2f730ff897ea-use_declaration-9"/><storycode:paste select="#f-2f730ff897ea-use_declaration-10"/><storycode:paste select="#f-2f730ff897ea-use_declaration-11"/><storycode:paste select="#f-2f730ff897ea-use_declaration-12"/><storycode:paste select="#f-2f730ff897ea-use_declaration-13"/><storycode:paste select="#f-2f730ff897ea-use_declaration-14"/><storycode:paste select="#f-2f730ff897ea-use_declaration-15"/><storycode:paste select="#f-2f730ff897ea-use_declaration-16"/><storycode:paste select="#f-2f730ff897ea-use_declaration-17"/><storycode:paste select="#f-2f730ff897ea-use_declaration-18"/><storycode:paste select="#f-2f730ff897ea-use_declaration-19"/></storycode:file>

</storycode:doc>
