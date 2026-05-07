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
pub mod dispatcher;
pub mod doc_store;
pub mod message_parser;
pub mod namespace;
pub mod node_bridge;
pub mod plugin;
pub mod plugins;
pub mod reactive_handler;
pub mod yrs_bridge;

use std::collections::HashMap;
use std::sync::Arc;

use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{ReadTxn, Text, Transact, Update, WriteTxn, Xml, XmlFragment, XmlFragmentRef};

use command::{CommandResult, GroveCommand};
use dispatcher::Dispatcher;
use doc_store::DocStore;
use message_parser::MessageNode;
use namespace::{DocId, NamespaceUri};
use plugin::NamespaceRegistry;
use tokio::sync::mpsc;
use yrs_bridge::YrsBridge;

/// Configuration for the GroveEngine.
pub struct GroveConfig {
    pub registry: NamespaceRegistry,
}

/// The core engine that manages documents, parses messages, and dispatches
/// change events to namespace handlers.
pub struct GroveEngine {
    doc_store: Arc<DocStore>,
    registry: Arc<NamespaceRegistry>,
}

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
