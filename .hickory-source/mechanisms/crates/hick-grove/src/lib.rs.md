<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-grove/src/lib.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-84d4eadc6a68"></a>
## `crates/hick-grove/src/lib.rs`

<a id="f-84d4eadc6a68-command"></a>
### `command`

Embeddable reactive XML document engine.
hick-grove implements the structured messages pattern: multi-namespace XML
messages are parsed, applied to a Yrs CRDT document, and the resulting
change events are dispatched to namespace-specific handlers that project
state into external stores (SQLite, search indices, etc.).
# Architecture
\```text
XML message ──► MessageParser ──► Yrs Doc (CRDT)
│
YrsBridge (observe_deep)
│
Dispatcher ──► NamespaceHandler(s)
│
Side effects
(DB writes, logs, …)
\```
# Key types
- [`GroveEngine`] — top-level entry point that owns the document store and
handler registry.
- [`command::GroveCommand`] — either a structured XML message or a raw Yrs
binary update.
- [`plugin::NamespaceHandler`] — trait for event-driven handlers keyed by
namespace URI.
- [`plugin::NamespaceRegistry`] — registry mapping namespace URIs to handlers.
- [`node_bridge::YrsElementNode`] — bridges a Yrs element into hick-flow's
reactive `Node` system via `tokio::sync::watch`.
# Example
\```rust,ignore
use std::sync::Arc;
use hick_grove::{GroveConfig, GroveEngine};
use hick_grove::namespace::NamespaceUri;
use hick_grove::plugin::NamespaceRegistry;
use hick_grove::command::GroveCommand;
let registry = NamespaceRegistry::new();
// registry.register(Arc::new(my_handler));
let engine = GroveEngine::new(GroveConfig { registry });
let doc_id = engine.create_document(vec![
("task".into(), NamespaceUri::new("https://example.com/vocab/task#")),
]).await;
let result = engine.process_command(GroveCommand::StructuredMessage {
doc_id: doc_id.clone(),
xml: r#"<msg xmlns:task="https://example.com/vocab/task#">
<task:create id="t-1" description="Buy milk"/>
</msg>"#.into(),
}).await?;
println!("{} events dispatched", result.events_dispatched);
\```

<storycode:copy id="f-84d4eadc6a68-command">
//! Embeddable reactive XML document engine.
//!
//! hick-grove implements the structured messages pattern: multi-namespace XML
//! messages are parsed, applied to a Yrs CRDT document, and the resulting
//! change events are dispatched to namespace-specific handlers that project
//! state into external stores (SQLite, search indices, etc.).
//!
//! # Architecture
//!
//! ```text
//! XML message ──► MessageParser ──► Yrs Doc (CRDT)
//!                                       │
//!                                  YrsBridge (observe_deep)
//!                                       │
//!                                  Dispatcher ──► NamespaceHandler(s)
//!                                                      │
//!                                                 Side effects
//!                                              (DB writes, logs, …)
//! ```
//!
//! # Key types
//!
//! - [`GroveEngine`] — top-level entry point that owns the document store and
//!   handler registry.
//! - [`command::GroveCommand`] — either a structured XML message or a raw Yrs
//!   binary update.
//! - [`plugin::NamespaceHandler`] — trait for event-driven handlers keyed by
//!   namespace URI.
//! - [`plugin::NamespaceRegistry`] — registry mapping namespace URIs to handlers.
//! - [`node_bridge::YrsElementNode`] — bridges a Yrs element into hick-flow's
//!   reactive [`Node`](hick_flow::Node) system via `tokio::sync::watch`.
//!
//! # Example
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use hick_grove::{GroveConfig, GroveEngine};
//! use hick_grove::namespace::NamespaceUri;
//! use hick_grove::plugin::NamespaceRegistry;
//! use hick_grove::command::GroveCommand;
//!
//! let registry = NamespaceRegistry::new();
//! // registry.register(Arc::new(my_handler));
//!
//! let engine = GroveEngine::new(GroveConfig { registry });
//!
//! let doc_id = engine.create_document(vec![
//!     ("task".into(), NamespaceUri::new("https://example.com/vocab/task#")),
//! ]).await;
//!
//! let result = engine.process_command(GroveCommand::StructuredMessage {
//!     doc_id: doc_id.clone(),
//!     xml: r#"<msg xmlns:task="https://example.com/vocab/task#">
//!               <task:create id="t-1" description="Buy milk"/>
//!             </msg>"#.into(),
//! }).await?;
//!
//! println!("{} events dispatched", result.events_dispatched);
//! ```

pub mod command;
</storycode:copy>
<a id="f-84d4eadc6a68-dispatcher"></a>
### `dispatcher`

<storycode:copy id="f-84d4eadc6a68-dispatcher">
pub mod dispatcher;
</storycode:copy>
<a id="f-84d4eadc6a68-doc_store"></a>
### `doc_store`

<storycode:copy id="f-84d4eadc6a68-doc_store">
pub mod doc_store;
</storycode:copy>
<a id="f-84d4eadc6a68-message_parser"></a>
### `message_parser`

<storycode:copy id="f-84d4eadc6a68-message_parser">
pub mod message_parser;
</storycode:copy>
<a id="f-84d4eadc6a68-namespace"></a>
### `namespace`

<storycode:copy id="f-84d4eadc6a68-namespace">
pub mod namespace;
</storycode:copy>
<a id="f-84d4eadc6a68-node_bridge"></a>
### `node_bridge`

<storycode:copy id="f-84d4eadc6a68-node_bridge">
pub mod node_bridge;
</storycode:copy>
<a id="f-84d4eadc6a68-plugin"></a>
### `plugin`

<storycode:copy id="f-84d4eadc6a68-plugin">
pub mod plugin;
</storycode:copy>
<a id="f-84d4eadc6a68-plugins"></a>
### `plugins`

<storycode:copy id="f-84d4eadc6a68-plugins">
pub mod plugins;
</storycode:copy>
<a id="f-84d4eadc6a68-reactive_handler"></a>
### `reactive_handler`

<storycode:copy id="f-84d4eadc6a68-reactive_handler">
pub mod reactive_handler;
</storycode:copy>
<a id="f-84d4eadc6a68-yrs_bridge"></a>
### `yrs_bridge`

<storycode:copy id="f-84d4eadc6a68-yrs_bridge">
pub mod yrs_bridge;

</storycode:copy>
<a id="f-84d4eadc6a68-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-84d4eadc6a68-use_declaration">
use std::collections::HashMap;
</storycode:copy>
<a id="f-84d4eadc6a68-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-84d4eadc6a68-use_declaration-2">
use std::sync::Arc;

</storycode:copy>
<a id="f-84d4eadc6a68-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-84d4eadc6a68-use_declaration-3">
use yrs::updates::decoder::Decode;
</storycode:copy>
<a id="f-84d4eadc6a68-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-84d4eadc6a68-use_declaration-4">
use yrs::updates::encoder::Encode;
</storycode:copy>
<a id="f-84d4eadc6a68-use_declaration-5"></a>
### `use_declaration`

<storycode:copy id="f-84d4eadc6a68-use_declaration-5">
use yrs::{ReadTxn, Text, Transact, Update, WriteTxn, Xml, XmlFragment, XmlFragmentRef};

</storycode:copy>
<a id="f-84d4eadc6a68-use_declaration-6"></a>
### `use_declaration`

<storycode:copy id="f-84d4eadc6a68-use_declaration-6">
use command::{CommandResult, GroveCommand};
</storycode:copy>
<a id="f-84d4eadc6a68-use_declaration-7"></a>
### `use_declaration`

<storycode:copy id="f-84d4eadc6a68-use_declaration-7">
use dispatcher::Dispatcher;
</storycode:copy>
<a id="f-84d4eadc6a68-use_declaration-8"></a>
### `use_declaration`

<storycode:copy id="f-84d4eadc6a68-use_declaration-8">
use doc_store::DocStore;
</storycode:copy>
<a id="f-84d4eadc6a68-use_declaration-9"></a>
### `use_declaration`

<storycode:copy id="f-84d4eadc6a68-use_declaration-9">
use message_parser::MessageNode;
</storycode:copy>
<a id="f-84d4eadc6a68-use_declaration-10"></a>
### `use_declaration`

<storycode:copy id="f-84d4eadc6a68-use_declaration-10">
use namespace::{DocId, NamespaceUri};
</storycode:copy>
<a id="f-84d4eadc6a68-use_declaration-11"></a>
### `use_declaration`

<storycode:copy id="f-84d4eadc6a68-use_declaration-11">
use plugin::NamespaceRegistry;
</storycode:copy>
<a id="f-84d4eadc6a68-use_declaration-12"></a>
### `use_declaration`

<storycode:copy id="f-84d4eadc6a68-use_declaration-12">
use tokio::sync::mpsc;
</storycode:copy>
<a id="f-84d4eadc6a68-use_declaration-13"></a>
### `use_declaration`

<storycode:copy id="f-84d4eadc6a68-use_declaration-13">
use yrs_bridge::YrsBridge;

</storycode:copy>
<a id="f-84d4eadc6a68-GroveConfig"></a>
### `GroveConfig`

<storycode:copy id="f-84d4eadc6a68-GroveConfig">
/// Configuration for the GroveEngine.
pub struct GroveConfig {
    pub registry: NamespaceRegistry,
}

</storycode:copy>
<a id="f-84d4eadc6a68-GroveEngine"></a>
### `GroveEngine`

The core engine that manages documents, parses messages, and dispatches
change events to namespace handlers.

<storycode:copy id="f-84d4eadc6a68-GroveEngine">
/// The core engine that manages documents, parses messages, and dispatches
/// change events to namespace handlers.
pub struct GroveEngine {
    doc_store: Arc<DocStore>,
    registry: Arc<NamespaceRegistry>,
}

</storycode:copy>
<a id="f-84d4eadc6a68-impl-GroveEngine"></a>
### `impl-GroveEngine`

<storycode:copy id="f-84d4eadc6a68-impl-GroveEngine">
impl GroveEngine {
    pub fn new(config: GroveConfig) -> Self {
        Self {
            doc_store: Arc::new(DocStore::new()),
            registry: Arc::new(config.registry),
        }
    }

    /// Create a new document with the given namespace declarations.
    pub async fn create_document(&self, namespaces: Vec<(String, NamespaceUri)>) -> DocId {
        let doc_id = DocId::new();
        let ns_map: HashMap<String, NamespaceUri> = namespaces.into_iter().collect();
        self.doc_store.create(doc_id.clone(), ns_map);
        doc_id
    }

    /// Process a command (structured message or binary update).
    pub async fn process_command(&self, command: GroveCommand) -> anyhow::Result<CommandResult> {
        match command {
            GroveCommand::StructuredMessage { doc_id, xml } => self.process_message(&doc_id, &xml),
            GroveCommand::BinaryUpdate { doc_id, update } => {
                self.apply_binary_update(&doc_id, &update)
            }
        }
    }

    /// Process a structured XML message against a document.
    fn process_message(&self, doc_id: &DocId, xml: &str) -> anyhow::Result<CommandResult> {
        let meta = self
            .doc_store
            .get(doc_id)
            .ok_or_else(|| anyhow::anyhow!("Document not found: {}", doc_id))?;

        let parsed = message_parser::parse_message(xml)?;

        // Attach bridge with a fresh channel to capture events from this txn
        let (tx, rx) = mpsc::unbounded_channel();
        let _bridge = YrsBridge::attach(
            &meta.doc,
            meta.namespaces.clone(),
            tx,
            meta.element_cache.clone(),
        );

        // Apply parsed message to the Yrs document
        let mut txn = meta.doc.transact_mut();
        let root: XmlFragmentRef = txn.get_or_insert_xml_fragment("root");
        apply_message_nodes(&mut txn, &root, &parsed.nodes, &parsed.namespaces);
        drop(txn);
        // Bridge subscription dropped here — that's fine, events were already sent synchronously

        // Drain the channel — events were pushed during the transaction
        let mut dispatcher = Dispatcher::new(self.registry.clone(), rx, doc_id.clone());
        let effects = dispatcher.drain_pending();

        Ok(CommandResult {
            doc_id: doc_id.clone(),
            events_dispatched: effects.len(),
        })
    }

    /// Apply a raw Yrs binary update to a document.
    fn apply_binary_update(&self, doc_id: &DocId, update: &[u8]) -> anyhow::Result<CommandResult> {
        let meta = self
            .doc_store
            .get(doc_id)
            .ok_or_else(|| anyhow::anyhow!("Document not found: {}", doc_id))?;

        let (tx, rx) = mpsc::unbounded_channel();
        let _bridge = YrsBridge::attach(
            &meta.doc,
            meta.namespaces.clone(),
            tx,
            meta.element_cache.clone(),
        );

        let mut txn = meta.doc.transact_mut();
        txn.apply_update(Update::decode_v1(update)?)?;
        drop(txn);

        let mut dispatcher = Dispatcher::new(self.registry.clone(), rx, doc_id.clone());
        let effects = dispatcher.drain_pending();

        Ok(CommandResult {
            doc_id: doc_id.clone(),
            events_dispatched: effects.len(),
        })
    }

    /// Get the Yrs state vector for sync.
    pub fn state_vector(&self, doc_id: &DocId) -> anyhow::Result<Vec<u8>> {
        let meta = self
            .doc_store
            .get(doc_id)
            .ok_or_else(|| anyhow::anyhow!("Document not found: {}", doc_id))?;
        let txn = meta.doc.transact();
        Ok(txn.state_vector().encode_v1())
    }

    /// Encode the diff from a remote state vector.
    pub fn encode_diff(&self, doc_id: &DocId, remote_sv: &[u8]) -> anyhow::Result<Vec<u8>> {
        let meta = self
            .doc_store
            .get(doc_id)
            .ok_or_else(|| anyhow::anyhow!("Document not found: {}", doc_id))?;
        let txn = meta.doc.transact();
        let sv = yrs::StateVector::decode_v1(remote_sv)?;
        Ok(txn.encode_diff_v1(&sv))
    }

    pub fn doc_store(&self) -> &Arc<DocStore> {
        &self.doc_store
    }

    pub fn registry(&self) -> &Arc<NamespaceRegistry> {
        &self.registry
    }
}

</storycode:copy>
<a id="f-84d4eadc6a68-apply_message_nodes"></a>
### `apply_message_nodes`

Apply parsed message nodes to a Yrs XmlFragment within a transaction.

<storycode:copy id="f-84d4eadc6a68-apply_message_nodes">
/// Apply parsed message nodes to a Yrs XmlFragment within a transaction.
fn apply_message_nodes(
    txn: &mut yrs::TransactionMut<'_>,
    parent: &XmlFragmentRef,
    nodes: &[MessageNode],
    namespaces: &HashMap<String, NamespaceUri>,
) {
    for node in nodes {
        match node {
            MessageNode::Text(text) => {
                let text_ref = parent.insert(txn, parent.len(txn), yrs::XmlTextPrelim::new(""));
                text_ref.push(txn, text);
            }
            MessageNode::Element {
                namespace_uri,
                local_name,
                attributes,
                children,
            } => {
                let prefix = namespaces
                    .iter()
                    .find(|(_, uri)| *uri == namespace_uri)
                    .map(|(p, _)| p.as_str())
                    .unwrap_or("");

                let tag = if prefix.is_empty() {
                    local_name.clone()
                } else {
                    format!("{}:{}", prefix, local_name)
                };

                let elem = parent.insert(txn, parent.len(txn), yrs::XmlElementPrelim::empty(tag));

                for (key, value) in attributes {
                    elem.insert_attribute(txn, key.as_str(), value.as_str());
                }

                apply_children_to_element(txn, &elem, children, namespaces);
            }
        }
    }
}

</storycode:copy>
<a id="f-84d4eadc6a68-apply_children_to_element"></a>
### `apply_children_to_element`

<storycode:copy id="f-84d4eadc6a68-apply_children_to_element">
fn apply_children_to_element(
    txn: &mut yrs::TransactionMut<'_>,
    parent: &yrs::XmlElementRef,
    children: &[MessageNode],
    namespaces: &HashMap<String, NamespaceUri>,
) {
    for child in children {
        match child {
            MessageNode::Text(text) => {
                let text_ref = parent.insert(txn, parent.len(txn), yrs::XmlTextPrelim::new(""));
                text_ref.push(txn, text);
            }
            MessageNode::Element {
                namespace_uri,
                local_name,
                attributes,
                children: grandchildren,
            } => {
                let prefix = namespaces
                    .iter()
                    .find(|(_, uri)| *uri == namespace_uri)
                    .map(|(p, _)| p.as_str())
                    .unwrap_or("");

                let tag = if prefix.is_empty() {
                    local_name.clone()
                } else {
                    format!("{}:{}", prefix, local_name)
                };

                let elem = parent.insert(txn, parent.len(txn), yrs::XmlElementPrelim::empty(tag));

                for (key, value) in attributes {
                    elem.insert_attribute(txn, key.as_str(), value.as_str());
                }

                apply_children_to_element(txn, &elem, grandchildren, namespaces);
            }
        }
    }
}
</storycode:copy>

### Assemble `crates/hick-grove/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-grove/src/lib.rs"><storycode:paste select="#f-84d4eadc6a68-command"/><storycode:paste select="#f-84d4eadc6a68-dispatcher"/><storycode:paste select="#f-84d4eadc6a68-doc_store"/><storycode:paste select="#f-84d4eadc6a68-message_parser"/><storycode:paste select="#f-84d4eadc6a68-namespace"/><storycode:paste select="#f-84d4eadc6a68-node_bridge"/><storycode:paste select="#f-84d4eadc6a68-plugin"/><storycode:paste select="#f-84d4eadc6a68-plugins"/><storycode:paste select="#f-84d4eadc6a68-reactive_handler"/><storycode:paste select="#f-84d4eadc6a68-yrs_bridge"/><storycode:paste select="#f-84d4eadc6a68-use_declaration"/><storycode:paste select="#f-84d4eadc6a68-use_declaration-2"/><storycode:paste select="#f-84d4eadc6a68-use_declaration-3"/><storycode:paste select="#f-84d4eadc6a68-use_declaration-4"/><storycode:paste select="#f-84d4eadc6a68-use_declaration-5"/><storycode:paste select="#f-84d4eadc6a68-use_declaration-6"/><storycode:paste select="#f-84d4eadc6a68-use_declaration-7"/><storycode:paste select="#f-84d4eadc6a68-use_declaration-8"/><storycode:paste select="#f-84d4eadc6a68-use_declaration-9"/><storycode:paste select="#f-84d4eadc6a68-use_declaration-10"/><storycode:paste select="#f-84d4eadc6a68-use_declaration-11"/><storycode:paste select="#f-84d4eadc6a68-use_declaration-12"/><storycode:paste select="#f-84d4eadc6a68-use_declaration-13"/><storycode:paste select="#f-84d4eadc6a68-GroveConfig"/><storycode:paste select="#f-84d4eadc6a68-GroveEngine"/><storycode:paste select="#f-84d4eadc6a68-impl-GroveEngine"/><storycode:paste select="#f-84d4eadc6a68-apply_message_nodes"/><storycode:paste select="#f-84d4eadc6a68-apply_children_to_element"/></storycode:file>

</storycode:doc>
