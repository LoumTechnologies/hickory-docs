<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Combine changing streams

We need to combine the latest values from a changing set of async streams.
The caller supplies the value types and combining function; this library does
not know documents, files, context or provenance. Hickory builds its node graph
in hick-exec using this interface.

<storycode:entry path="crates/hick-flow/src/lib.rs">
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
</storycode:entry>
</storycode:doc>
