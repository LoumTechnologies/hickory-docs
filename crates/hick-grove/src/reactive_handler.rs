use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::namespace::NamespaceUri;
use crate::node_bridge::{ElementIdentity, ElementSnapshot, ElementSnapshotSender, YrsElementNode};
use crate::plugin::{ElementEvent, HandlerContext, NamespaceHandler, SideEffect};

struct TrackedElement {
    sender: ElementSnapshotSender,
    snapshot: ElementSnapshot,
}

/// A `NamespaceHandler` that bridges element events into reactive
/// `YrsElementNode` streams. Callers subscribe by identity; matching events
/// push updated `ElementSnapshot`s through `tokio::sync::watch` channels.
pub struct ReactiveNamespaceHandler {
    namespace_uri: NamespaceUri,
    tracked: Mutex<HashMap<ElementIdentity, TrackedElement>>,
}

impl ReactiveNamespaceHandler {
    pub fn new(namespace_uri: NamespaceUri) -> Self {
        Self {
            namespace_uri,
            tracked: Mutex::new(HashMap::new()),
        }
    }

    /// Subscribe to a specific element identity. Returns a `YrsElementNode`
    /// whose stream will emit snapshots as events arrive.
    ///
    /// The returned node immediately holds a default (empty) snapshot.
    /// When a `Created` event matching the identity arrives, the first real
    /// snapshot is pushed; subsequent `AttributeChanged`/`TextChanged` events
    /// update the snapshot incrementally. A `Removed` event drops the sender,
    /// terminating the watch stream.
    pub fn subscribe(&self, identity: ElementIdentity) -> Arc<YrsElementNode> {
        let initial = ElementSnapshot {
            local_name: identity.local_name.clone(),
            attributes: vec![],
            text_content: String::new(),
        };

        let (sender, rx) = ElementSnapshotSender::new(initial.clone());
        let node = Arc::new(YrsElementNode::new(rx));

        let mut tracked = self.tracked.lock().unwrap();
        tracked.insert(
            identity,
            TrackedElement {
                sender,
                snapshot: initial,
            },
        );

        node
    }

    /// Check whether a given identity is currently subscribed.
    pub fn is_subscribed(&self, identity: &ElementIdentity) -> bool {
        self.tracked.lock().unwrap().contains_key(identity)
    }
}

impl NamespaceHandler for ReactiveNamespaceHandler {
    fn namespace_uri(&self) -> &NamespaceUri {
        &self.namespace_uri
    }

    fn name(&self) -> &str {
        "reactive"
    }

    fn handle_event(
        &self,
        event: &ElementEvent,
        _ctx: &HandlerContext,
    ) -> anyhow::Result<Vec<SideEffect>> {
        match event {
            ElementEvent::Created {
                local_name,
                attributes,
                text_content,
                element_id,
            } => {
                if let Some(id) = element_id {
                    let identity = ElementIdentity {
                        namespace_uri: self.namespace_uri.clone(),
                        local_name: local_name.clone(),
                        id: id.clone(),
                    };
                    let mut tracked = self.tracked.lock().unwrap();
                    if let Some(entry) = tracked.get_mut(&identity) {
                        let snap = ElementSnapshot {
                            local_name: local_name.clone(),
                            attributes: attributes.clone(),
                            text_content: text_content.clone(),
                        };
                        entry.sender.send(snap.clone());
                        entry.snapshot = snap;
                    }
                }
            }

            ElementEvent::AttributeChanged {
                local_name,
                attr_name,
                new,
                element_id,
                ..
            } => {
                if let Some(id) = element_id {
                    let identity = ElementIdentity {
                        namespace_uri: self.namespace_uri.clone(),
                        local_name: local_name.clone(),
                        id: id.clone(),
                    };
                    let mut tracked = self.tracked.lock().unwrap();
                    if let Some(entry) = tracked.get_mut(&identity) {
                        // Update attribute in current snapshot
                        let attrs = &mut entry.snapshot.attributes;
                        if let Some(new_val) = new {
                            if let Some(existing) = attrs.iter_mut().find(|(k, _)| k == attr_name) {
                                existing.1 = new_val.clone();
                            } else {
                                attrs.push((attr_name.clone(), new_val.clone()));
                            }
                        } else {
                            attrs.retain(|(k, _)| k != attr_name);
                        }
                        entry.sender.send(entry.snapshot.clone());
                    }
                }
            }

            ElementEvent::TextChanged {
                local_name,
                new_text,
                element_id,
            } => {
                if let Some(id) = element_id {
                    let identity = ElementIdentity {
                        namespace_uri: self.namespace_uri.clone(),
                        local_name: local_name.clone(),
                        id: id.clone(),
                    };
                    let mut tracked = self.tracked.lock().unwrap();
                    if let Some(entry) = tracked.get_mut(&identity) {
                        entry.snapshot.text_content = new_text.clone();
                        entry.sender.send(entry.snapshot.clone());
                    }
                }
            }

            ElementEvent::Removed {
                local_name,
                element_id,
                ..
            } => {
                if let Some(id) = element_id {
                    let identity = ElementIdentity {
                        namespace_uri: self.namespace_uri.clone(),
                        local_name: local_name.clone(),
                        id: id.clone(),
                    };
                    // Remove from tracked — dropping the sender closes the watch channel
                    self.tracked.lock().unwrap().remove(&identity);
                }
            }
        }

        Ok(vec![])
    }
}
