//! Data source trait and implementations.

pub mod csv;
pub mod memory;

use hick_flow::Context;

use crate::changes::ChangeStream;
use crate::schema::LiveSchema;

/// Trait for sources that produce a `ChangeStream`.
///
/// Every source starts by emitting an Add event with the initial state,
/// then emits subsequent Add/Remove events as the underlying data changes.
pub trait ChangeSource: Send + Sync {
    /// The schema of records produced by this source.
    fn schema(&self) -> &LiveSchema;

    /// Produce a stream of change events.
    fn stream(&self, context: Context) -> ChangeStream;
}
