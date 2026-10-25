<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-flow/src/context.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `b52c28c8091504205ec1239c41a6c054dd0e28165fdc7ed26f4d07082e0253cf`.

<book:entry path="crates/hick-flow/src/context.rs">
//! Execution context with type-erased extension map.
//!
//! `Context` is passed to every `Node::get_stream` call. Domain-specific state
//! (e.g. `MultiDocumentState`) can be stored as an extension using the TypeMap
//! pattern, keeping this crate free of domain dependencies.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// Execution context passed to all nodes.
#[derive(Clone, Default)]
pub struct Context {
    extensions: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    pub verbose: bool,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return a new context with the given extension added.
    pub fn with_extension<T: Send + Sync + 'static>(mut self, value: T) -> Self {
        self.extensions.insert(TypeId::of::<T>(), Arc::new(value));
        self
    }

    /// Add an extension to an existing context.
    pub fn set_extension<T: Send + Sync + 'static>(&mut self, value: T) {
        self.extensions.insert(TypeId::of::<T>(), Arc::new(value));
    }

    /// Retrieve a previously stored extension by type.
    pub fn get_extension<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.extensions
            .get(&TypeId::of::<T>())
            .and_then(|v| Arc::clone(v).downcast::<T>().ok())
    }
}
</book:entry>

</book:doc>
