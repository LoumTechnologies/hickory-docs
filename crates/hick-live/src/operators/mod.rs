//! Set operators for incremental collection processing.

pub mod group_by;
pub mod select;
pub mod select_many;
pub mod where_op;

pub use group_by::GroupByNode;
pub use select::SelectNode;
pub use select_many::SelectManyNode;
pub use where_op::WhereNode;
