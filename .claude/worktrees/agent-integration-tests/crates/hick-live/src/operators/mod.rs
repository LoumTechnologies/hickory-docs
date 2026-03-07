//! Set operators for incremental collection processing.

pub mod group_by;
pub mod mask;
pub mod select;
pub mod select_many;
pub mod subscribe;
pub mod where_op;

pub use group_by::GroupByNode;
pub use mask::MaskNode;
pub use select::SelectNode;
pub use select_many::SelectManyNode;
pub use subscribe::SubscribeNode;
pub use where_op::WhereNode;
