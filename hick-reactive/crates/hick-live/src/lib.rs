//! Incremental collection processing for hick pipelines.
//!
//! Ports the LiveLinq pattern to Rust: set-based operators (Where, Select,
//! GroupBy, SelectMany) over Arrow RecordBatch streams, integrated with
//! hick-flow's reactive Node system.

pub mod changes;
pub mod handlers;
pub mod node;
pub mod operators;
pub mod output;
pub mod request;
pub mod schema;
pub mod sources;

pub use changes::strict::make_strict;
pub use changes::{ChangeStream, ChangeType, RecordChange};
pub use node::{NodeExt, RecordChangeNode, StreamingStructuredDataNode};
pub use schema::LiveSchema;
pub use sources::ChangeSource;
