use std::collections::HashMap;
use std::sync::Arc;

use crate::namespace::{DocId, NamespaceUri};

/// An event describing a change to a namespace-qualified element.
#[derive(Clone, Debug)]
pub enum ElementEvent {
    Created {
        local_name: String,
        attributes: Vec<(String, String)>,
        text_content: String,
        element_id: Option<String>,
    },
    AttributeChanged {
        local_name: String,
        attr_name: String,
        old: Option<String>,
        new: Option<String>,
        element_id: Option<String>,
    },
    TextChanged {
        local_name: String,
        new_text: String,
        element_id: Option<String>,
    },
    Removed {
        local_name: String,
        attributes: Vec<(String, String)>,
        element_id: Option<String>,
    },
}

/// Side effects produced by a handler in response to an event.
#[derive(Clone, Debug)]
pub enum SideEffect {
    Log(String),
    Custom(String),
}

/// Context passed to handlers alongside an event.
pub struct HandlerContext {
    pub doc_id: DocId,
}

/// Trait for namespace-specific event handlers.
///
/// Handlers are registered for a specific namespace URI and receive element
/// events when changes occur to elements in that namespace.
pub trait NamespaceHandler: Send + Sync {
    fn namespace_uri(&self) -> &NamespaceUri;
    fn name(&self) -> &str;
    fn handle_event(
        &self,
        event: &ElementEvent,
        ctx: &HandlerContext,
    ) -> anyhow::Result<Vec<SideEffect>>;
}

/// Registry of namespace handlers, keyed by namespace URI.
pub struct NamespaceRegistry {
    handlers: HashMap<NamespaceUri, Vec<Arc<dyn NamespaceHandler>>>,
}

impl Default for NamespaceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl NamespaceRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register(&mut self, handler: Arc<dyn NamespaceHandler>) {
        self.handlers
            .entry(handler.namespace_uri().clone())
            .or_default()
            .push(handler);
    }

    pub fn handlers_for(&self, ns: &NamespaceUri) -> &[Arc<dyn NamespaceHandler>] {
        self.handlers.get(ns).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn all_namespaces(&self) -> Vec<&NamespaceUri> {
        self.handlers.keys().collect()
    }
}
