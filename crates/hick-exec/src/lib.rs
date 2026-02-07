//! Execution engine for `.hick` files.
//!
//! Ports `DynamicCombineLatest`, `Node`, `InsertionPoint`, and related types from
//! HickoryDocs, and adds information-flow DAG construction and validation.

pub mod combine_latest;
pub mod dag;
pub mod node;
pub mod state;
pub mod volume;

pub use combine_latest::DynamicCombineLatest;
pub use dag::{DagEdge, DagValidationError, ExecId, FlowDag};
pub use node::{
    BoxStream, Context, InsertionPoint, Node, NodeTrace, SeparatorNode, StringNode, TransformNode,
    converge_to_string,
};
pub use state::MultiDocumentState;
