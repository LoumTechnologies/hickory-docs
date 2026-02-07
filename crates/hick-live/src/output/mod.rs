//! Output nodes for persisting change streams.

pub mod parquet;

pub use parquet::{ParquetFileNode, ParquetMode};
