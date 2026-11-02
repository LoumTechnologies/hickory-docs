//! Combine the latest values from a dynamic set of async streams.
//!
//! Callers supply streams and a combining function; document policy belongs
//! in the consuming application.

use futures::Stream;
use std::pin::Pin;

pub mod combine_latest;
pub use combine_latest::DynamicCombineLatest;

/// An owned, sendable async stream.
pub type BoxStream<T> = Pin<Box<dyn Stream<Item = T> + Send + 'static>>;
