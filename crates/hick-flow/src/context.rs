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
