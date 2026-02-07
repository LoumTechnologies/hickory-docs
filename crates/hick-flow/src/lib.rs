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
pub mod transform;

pub use combine_latest::DynamicCombineLatest;
pub use context::Context;
pub use converge::converge_to_string;
pub use insertion_point::InsertionPoint;
pub use node::{BoxStream, Node, NodeTrace, SeparatorNode, StringNode};
pub use transform::TransformNode;
