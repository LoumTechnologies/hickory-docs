//! XML tag handlers for hick-live operators.

mod data_source_handler;
mod group_by_handler;
mod parquet_file_handler;
mod select_handler;
mod select_many_handler;
mod where_handler;

pub use data_source_handler::DataSourceHandler;
pub use group_by_handler::GroupByHandler;
pub use parquet_file_handler::ParquetFileHandler;
pub use select_handler::SelectHandler;
pub use select_many_handler::SelectManyHandler;
pub use where_handler::WhereHandler;

use hick_handlers::TagRegistry;

/// Register all hick-live tag handlers with a registry.
pub fn register_live_builtins(registry: &mut TagRegistry) {
    registry.register(Box::new(WhereHandler));
    registry.register(Box::new(SelectHandler));
    registry.register(Box::new(GroupByHandler));
    registry.register(Box::new(SelectManyHandler));
    registry.register(Box::new(ParquetFileHandler));
    registry.register(Box::new(DataSourceHandler));
}
