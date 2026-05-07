//! Execution engine for `.hick` files.
//!
//! Ports `DynamicCombineLatest`, `Node`, `InsertionPoint`, and related types from
//! HickoryDocs, and adds information-flow DAG construction and validation.

pub mod combine_latest;
pub mod dag;
pub mod node;
pub mod state;
pub mod volume;

/// Combines multiple async streams, emitting the latest merged value whenever any source updates.
pub use combine_latest::DynamicCombineLatest;

/// An edge in the information-flow DAG representing a dependency between exec elements.
pub use dag::DagEdge;

/// Error produced when the information-flow DAG contains cycles or broken references.
pub use dag::DagValidationError;

/// Unique identifier for an `<hick:exec>` element, assigned by document order.
pub use dag::ExecId;

/// The validated information-flow DAG of exec elements, their edges, and root nodes.
pub use dag::FlowDag;

/// Type-erased pinned async `Stream` used throughout the node graph.
pub use node::BoxStream;

/// Per-evaluation context passed through the reactive node graph.
pub use node::Context;

/// Ordered collection of child nodes whose outputs are concatenated in sequence.
pub use node::InsertionPoint;

/// Core trait for reactive document nodes that produce streaming output.
pub use node::Node;

/// Trace record capturing which nodes contributed to a converged result.
pub use node::NodeTrace;

/// Node that emits a fixed separator string between sibling outputs.
pub use node::SeparatorNode;

/// Leaf node that emits a single static string value.
pub use node::StringNode;

/// Node that applies a mapping function to the output of a child node.
pub use node::TransformNode;

/// Converges a node graph into a single `String`, collecting all streamed segments.
pub use node::converge_to_string;

/// Global mutable state shared across a hick execution run, tracking files, copy/paste blocks, and variables.
pub use state::MultiDocumentState;
