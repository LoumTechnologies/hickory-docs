<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-grove/src/plugin.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b798a69f73b7"></a>
## `crates/hick-grove/src/plugin.rs`

<a id="f-b798a69f73b7-ElementEvent"></a>
### `ElementEvent`

<storycode:copy id="f-b798a69f73b7-ElementEvent">
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

</storycode:copy>
<a id="f-b798a69f73b7-SideEffect"></a>
### `SideEffect`

Side effects produced by a handler in response to an event.

<storycode:copy id="f-b798a69f73b7-SideEffect">
/// Side effects produced by a handler in response to an event.
#[derive(Clone, Debug)]
pub enum SideEffect {
    Log(String),
    Custom(String),
}

</storycode:copy>
<a id="f-b798a69f73b7-HandlerContext"></a>
### `HandlerContext`

Context passed to handlers alongside an event.

<storycode:copy id="f-b798a69f73b7-HandlerContext">
/// Context passed to handlers alongside an event.
pub struct HandlerContext {
    pub doc_id: DocId,
}

</storycode:copy>
<a id="f-b798a69f73b7-NamespaceHandler"></a>
### `NamespaceHandler`

Trait for namespace-specific event handlers.
Handlers are registered for a specific namespace URI and receive element
events when changes occur to elements in that namespace.

<storycode:copy id="f-b798a69f73b7-NamespaceHandler">
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

</storycode:copy>
<a id="f-b798a69f73b7-NamespaceRegistry"></a>
### `NamespaceRegistry`

Registry of namespace handlers, keyed by namespace URI.

<storycode:copy id="f-b798a69f73b7-NamespaceRegistry">
/// Registry of namespace handlers, keyed by namespace URI.
pub struct NamespaceRegistry {
    handlers: HashMap<NamespaceUri, Vec<Arc<dyn NamespaceHandler>>>,
}

</storycode:copy>
<a id="f-b798a69f73b7-impl-Default-for-NamespaceRegistry"></a>
### `impl-Default for NamespaceRegistry`

<storycode:copy id="f-b798a69f73b7-impl-Default-for-NamespaceRegistry">
impl Default for NamespaceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

</storycode:copy>
<a id="f-b798a69f73b7-impl-NamespaceRegistry"></a>
### `impl-NamespaceRegistry`

<storycode:copy id="f-b798a69f73b7-impl-NamespaceRegistry">
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
</storycode:copy>

### Assemble `crates/hick-grove/src/plugin.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-grove/src/plugin.rs"><storycode:paste select="#f-b798a69f73b7-ElementEvent"/><storycode:paste select="#f-b798a69f73b7-SideEffect"/><storycode:paste select="#f-b798a69f73b7-HandlerContext"/><storycode:paste select="#f-b798a69f73b7-NamespaceHandler"/><storycode:paste select="#f-b798a69f73b7-NamespaceRegistry"/><storycode:paste select="#f-b798a69f73b7-impl-Default-for-NamespaceRegistry"/><storycode:paste select="#f-b798a69f73b7-impl-NamespaceRegistry"/></storycode:file>

</storycode:doc>
