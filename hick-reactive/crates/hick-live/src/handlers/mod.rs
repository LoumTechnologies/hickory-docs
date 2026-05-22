//! XML tag handlers for hick-live operators.

mod data_source_handler;
mod group_by_handler;
mod mask_handler;
mod parquet_file_handler;
mod select_handler;
mod select_many_handler;
mod subscribe_handler;
mod where_handler;

pub use data_source_handler::DataSourceHandler;
pub use group_by_handler::GroupByHandler;
pub use mask_handler::MaskHandler;
pub use parquet_file_handler::ParquetFileHandler;
pub use select_handler::SelectHandler;
pub use select_many_handler::SelectManyHandler;
pub use subscribe_handler::SubscribeHandler;
pub use where_handler::WhereHandler;

use std::sync::Arc;

use hick_handlers::TagRegistry;

use crate::schema::LiveSchema;

/// Register all hick-live tag handlers with a registry.
pub fn register_live_builtins(registry: &mut TagRegistry) {
    registry.register(Box::new(WhereHandler));
    registry.register(Box::new(SelectHandler));
    registry.register(Box::new(GroupByHandler));
    registry.register(Box::new(SelectManyHandler));
    registry.register(Box::new(ParquetFileHandler));
    registry.register(Box::new(DataSourceHandler));
    registry.register(Box::new(MaskHandler));
    registry.register(Box::new(SubscribeHandler));
}

/// Infer a `LiveSchema` from a source node by downcasting to known types.
pub(crate) fn infer_schema(source: &Arc<dyn hick_flow::Node>) -> LiveSchema {
    if let Some(any) = source.as_any() {
        if let Some(n) = any.downcast_ref::<crate::node::StreamingStructuredDataNode>() {
            return n.schema.clone();
        }
        if let Some(n) = any.downcast_ref::<crate::node::RecordChangeNode>() {
            return LiveSchema::new_strict(n.change.records.schema(), vec![]);
        }
        if let Some(n) = any.downcast_ref::<crate::operators::where_op::WhereNode>() {
            return n.schema.clone();
        }
        if let Some(n) = any.downcast_ref::<crate::operators::select::SelectNode>() {
            return n.output_schema.clone();
        }
        if let Some(n) = any.downcast_ref::<crate::operators::group_by::GroupByNode>() {
            return n.output_schema.clone();
        }
        if let Some(n) = any.downcast_ref::<crate::operators::select_many::SelectManyNode>() {
            return n.output_schema.clone();
        }
        if let Some(n) = any.downcast_ref::<crate::operators::mask::MaskNode>() {
            return n.output_schema.clone();
        }
        if let Some(n) = any.downcast_ref::<crate::operators::subscribe::SubscribeNode>() {
            return n.output_schema.clone();
        }
    }
    // Fallback: empty schema
    LiveSchema::new_strict(Arc::new(arrow::datatypes::Schema::empty()), vec![])
}
