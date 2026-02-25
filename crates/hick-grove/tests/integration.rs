use std::collections::HashMap;
use std::sync::Arc;

use hick_grove::command::GroveCommand;
use hick_grove::message_parser::{self, MessageNode};
use hick_grove::namespace::{DocId, NamespaceUri};
use hick_grove::plugin::{
    ElementEvent, HandlerContext, NamespaceHandler, NamespaceRegistry, SideEffect,
};
use hick_grove::{GroveConfig, GroveEngine};

const TASK_NS: &str = "https://example.com/vocab/task#";

// -----------------------------------------------------------------------
// Phase 2: DocStore + Yrs bridge
// -----------------------------------------------------------------------

#[tokio::test]
async fn test_doc_store_create_and_get() {
    let store = hick_grove::doc_store::DocStore::new();
    let mut ns = HashMap::new();
    ns.insert("task".to_string(), NamespaceUri::new(TASK_NS));

    let doc_id = DocId::from_string("doc-test-1");
    let meta = store.create(doc_id.clone(), ns);
    assert!(store.contains(&doc_id));
    assert_eq!(meta.namespaces.len(), 1);
    assert_eq!(meta.namespaces.get("task").unwrap().as_str(), TASK_NS);

    let retrieved = store.get(&doc_id).unwrap();
    assert_eq!(retrieved.namespaces.len(), 1);
}

#[tokio::test]
async fn test_doc_store_missing_doc() {
    let store = hick_grove::doc_store::DocStore::new();
    let doc_id = DocId::from_string("nonexistent");
    assert!(store.get(&doc_id).is_none());
    assert!(!store.contains(&doc_id));
}

#[tokio::test]
async fn test_yrs_bridge_fires_events() {
    use hick_grove::yrs_bridge::{ElementCache, YrsBridge};
    use tokio::sync::mpsc;
    use yrs::{Doc, Text, Transact, WriteTxn, Xml, XmlFragment, XmlFragmentRef};

    let doc = Doc::new();
    let mut ns = HashMap::new();
    ns.insert("task".to_string(), NamespaceUri::new(TASK_NS));

    let (tx, mut rx) = mpsc::unbounded_channel();
    let _bridge = YrsBridge::attach(&doc, ns, tx, ElementCache::new());

    // Apply a transaction that inserts a namespaced element
    {
        let mut txn = doc.transact_mut();
        let root: XmlFragmentRef = txn.get_or_insert_xml_fragment("root");
        let elem = root.insert(&mut txn, 0, yrs::XmlElementPrelim::empty("task:create"));
        elem.insert_attribute(&mut txn, "id", "task-1");
        elem.insert_attribute(&mut txn, "description", "Change oil");
        let text = elem.insert(&mut txn, 0, yrs::XmlTextPrelim::new(""));
        text.push(&mut txn, "created a task");
    }

    // Events should have been pushed synchronously
    let mut events = Vec::new();
    while let Ok(ev) = rx.try_recv() {
        events.push(ev);
    }

    assert!(
        !events.is_empty(),
        "Should have received at least one event"
    );

    // At minimum, we should see a Created event for task:create
    let created = events.iter().find(
        |e| matches!(&e.event, ElementEvent::Created { local_name, .. } if local_name == "create"),
    );
    assert!(
        created.is_some(),
        "Should have a Created event for 'create'"
    );
    assert_eq!(created.unwrap().namespace_uri.as_str(), TASK_NS);
}

// -----------------------------------------------------------------------
// Phase 3: Message parser
// -----------------------------------------------------------------------

#[test]
fn test_parse_simple_message() {
    let xml = r#"<message xmlns:task="https://example.com/vocab/task#">
        <task:create id="task-1" description="Change oil">created a task</task:create>
    </message>"#;

    let parsed = message_parser::parse_message(xml).unwrap();

    // Should have the task namespace
    assert_eq!(parsed.namespaces.get("task").unwrap().as_str(), TASK_NS);

    // Root element is "message"
    assert_eq!(parsed.nodes.len(), 1);
    if let MessageNode::Element {
        local_name,
        children,
        ..
    } = &parsed.nodes[0]
    {
        assert_eq!(local_name, "message");
        // Should have whitespace text + task:create element + whitespace
        let elements: Vec<_> = children
            .iter()
            .filter(|c| matches!(c, MessageNode::Element { .. }))
            .collect();
        assert_eq!(elements.len(), 1);

        if let MessageNode::Element {
            namespace_uri,
            local_name,
            attributes,
            children,
        } = &elements[0]
        {
            assert_eq!(namespace_uri.as_str(), TASK_NS);
            assert_eq!(local_name, "create");
            assert!(attributes.iter().any(|(k, v)| k == "id" && v == "task-1"));
            assert!(
                attributes
                    .iter()
                    .any(|(k, v)| k == "description" && v == "Change oil")
            );
            // Text content
            let texts: Vec<_> = children
                .iter()
                .filter_map(|c| {
                    if let MessageNode::Text(t) = c {
                        Some(t.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(texts, vec!["created a task"]);
        } else {
            panic!("Expected element");
        }
    } else {
        panic!("Expected root element");
    }
}

#[test]
fn test_parse_nested_namespaces() {
    let xml = r#"<message xmlns:task="https://example.com/vocab/task#" xmlns:user="https://example.com/vocab/user#">
        <task:create id="task-1" description="Change oil">
            created and <task:assign to="joe-1">assigned this to you</task:assign>.
        </task:create>
    </message>"#;

    let parsed = message_parser::parse_message(xml).unwrap();

    assert!(parsed.namespaces.contains_key("task"));
    assert!(parsed.namespaces.contains_key("user"));

    // Navigate to task:create → task:assign
    if let Some(MessageNode::Element { children, .. }) = parsed.nodes.first() {
        let create_elem = children
            .iter()
            .find(
                |c| matches!(c, MessageNode::Element { local_name, .. } if local_name == "create"),
            )
            .unwrap();

        if let MessageNode::Element { children, .. } = create_elem {
            let assign_elem = children
                .iter()
                .find(|c| {
                    matches!(c, MessageNode::Element { local_name, .. } if local_name == "assign")
                })
                .unwrap();

            if let MessageNode::Element {
                namespace_uri,
                attributes,
                children,
                ..
            } = assign_elem
            {
                assert_eq!(namespace_uri.as_str(), TASK_NS);
                assert!(attributes.iter().any(|(k, v)| k == "to" && v == "joe-1"));
                // Text content of task:assign
                assert!(
                    children
                        .iter()
                        .any(|c| matches!(c, MessageNode::Text(t) if t == "assigned this to you"))
                );
            }
        }
    }
}

#[test]
fn test_parse_empty_element() {
    let xml = r#"<root xmlns:ns="http://test.com"><ns:item id="1"/></root>"#;
    let parsed = message_parser::parse_message(xml).unwrap();

    if let Some(MessageNode::Element { children, .. }) = parsed.nodes.first() {
        if let Some(MessageNode::Element {
            local_name,
            attributes,
            children: inner,
            ..
        }) = children.first()
        {
            assert_eq!(local_name, "item");
            assert!(attributes.iter().any(|(k, v)| k == "id" && v == "1"));
            assert!(inner.is_empty());
        } else {
            panic!("Expected element");
        }
    }
}

// -----------------------------------------------------------------------
// Phase 4: Dispatcher + engine wiring
// -----------------------------------------------------------------------

/// A test handler that records all events it receives.
struct RecordingHandler {
    namespace: NamespaceUri,
    events: std::sync::Mutex<Vec<ElementEvent>>,
}

impl RecordingHandler {
    fn new(ns: &str) -> Self {
        Self {
            namespace: NamespaceUri::new(ns),
            events: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn recorded_events(&self) -> Vec<ElementEvent> {
        self.events.lock().unwrap().clone()
    }
}

impl NamespaceHandler for RecordingHandler {
    fn namespace_uri(&self) -> &NamespaceUri {
        &self.namespace
    }

    fn name(&self) -> &str {
        "recording"
    }

    fn handle_event(
        &self,
        event: &ElementEvent,
        _ctx: &HandlerContext,
    ) -> anyhow::Result<Vec<SideEffect>> {
        self.events.lock().unwrap().push(event.clone());
        Ok(vec![SideEffect::Log(format!("{:?}", event))])
    }
}

#[tokio::test]
async fn test_engine_create_doc_and_process_message() {
    let handler = Arc::new(RecordingHandler::new(TASK_NS));
    let handler_clone = handler.clone();

    let mut registry = NamespaceRegistry::new();
    registry.register(handler);

    let engine = GroveEngine::new(GroveConfig { registry });

    let doc_id = engine
        .create_document(vec![("task".to_string(), NamespaceUri::new(TASK_NS))])
        .await;

    let xml = r#"<message xmlns:task="https://example.com/vocab/task#">
        <task:create id="task-1" description="Change oil">created a task</task:create>
    </message>"#;

    let result = engine
        .process_command(GroveCommand::StructuredMessage {
            doc_id: doc_id.clone(),
            xml: xml.to_string(),
        })
        .await
        .unwrap();

    assert_eq!(result.doc_id, doc_id);

    // The handler should have received events
    let events = handler_clone.recorded_events();
    assert!(!events.is_empty(), "Handler should have received events");

    let created = events
        .iter()
        .find(|e| matches!(e, ElementEvent::Created { local_name, .. } if local_name == "create"));
    assert!(
        created.is_some(),
        "Should have received Created event for 'create'"
    );

    if let Some(ElementEvent::Created { attributes, .. }) = created {
        assert!(attributes.iter().any(|(k, v)| k == "id" && v == "task-1"));
        assert!(
            attributes
                .iter()
                .any(|(k, v)| k == "description" && v == "Change oil")
        );
    }
}

#[tokio::test]
async fn test_engine_process_nonexistent_doc() {
    let registry = NamespaceRegistry::new();
    let engine = GroveEngine::new(GroveConfig { registry });

    let result = engine
        .process_command(GroveCommand::StructuredMessage {
            doc_id: DocId::from_string("nonexistent"),
            xml: "<root/>".to_string(),
        })
        .await;

    assert!(result.is_err());
}

// -----------------------------------------------------------------------
// Phase 5: Tasks plugin (SQLite)
// -----------------------------------------------------------------------

#[cfg(feature = "sqlite-example")]
mod tasks_tests {
    use super::*;
    use hick_grove::plugins::tasks::{TasksHandler, TasksSqliteStore};

    #[tokio::test]
    async fn test_tasks_create_via_engine() {
        let store = Arc::new(TasksSqliteStore::new_in_memory().unwrap());
        let handler = Arc::new(TasksHandler::new(store.clone()));

        let mut registry = NamespaceRegistry::new();
        registry.register(handler);

        let engine = GroveEngine::new(GroveConfig { registry });

        let doc_id = engine
            .create_document(vec![("task".to_string(), NamespaceUri::new(TASK_NS))])
            .await;

        let xml = r#"<message xmlns:task="https://example.com/vocab/task#">
            <task:create id="task-1" description="Change oil">created a task</task:create>
        </message>"#;

        engine
            .process_command(GroveCommand::StructuredMessage {
                doc_id: doc_id.clone(),
                xml: xml.to_string(),
            })
            .await
            .unwrap();

        // Check the task was created in SQLite
        let task = store.get_task("task-1").unwrap();
        assert!(task.is_some(), "Task should exist in SQLite");
        let task = task.unwrap();
        assert_eq!(task.id, "task-1");
        assert_eq!(task.description, "Change oil");
        assert_eq!(task.doc_id, doc_id.as_str());
    }

    #[tokio::test]
    async fn test_tasks_store_list_by_doc() {
        let store = TasksSqliteStore::new_in_memory().unwrap();
        store.create_task("t1", "Task 1", "doc-a").unwrap();
        store.create_task("t2", "Task 2", "doc-a").unwrap();
        store.create_task("t3", "Task 3", "doc-b").unwrap();

        let tasks = store.list_tasks("doc-a").unwrap();
        assert_eq!(tasks.len(), 2);

        let tasks = store.list_tasks("doc-b").unwrap();
        assert_eq!(tasks.len(), 1);
    }

    #[tokio::test]
    async fn test_tasks_assign() {
        let store = TasksSqliteStore::new_in_memory().unwrap();
        store.create_task("t1", "Task 1", "doc-a").unwrap();
        store.assign_task("t1", "joe-1").unwrap();

        let task = store.get_task("t1").unwrap().unwrap();
        assert_eq!(task.assignee.as_deref(), Some("joe-1"));
    }

    #[tokio::test]
    async fn test_tasks_due_date() {
        let store = TasksSqliteStore::new_in_memory().unwrap();
        store.create_task("t1", "Task 1", "doc-a").unwrap();
        store.set_due_date("t1", "2025-06-15").unwrap();

        let task = store.get_task("t1").unwrap().unwrap();
        assert_eq!(task.due_date.as_deref(), Some("2025-06-15"));
    }
}

// -----------------------------------------------------------------------
// Phase 6: Node bridge
// -----------------------------------------------------------------------

#[tokio::test]
async fn test_yrs_element_node_emits_snapshots() {
    use futures::StreamExt;
    use hick_flow::{Context, Node};
    use hick_grove::node_bridge::{ElementSnapshot, ElementSnapshotSender, YrsElementNode};

    let initial = ElementSnapshot {
        local_name: "item".to_string(),
        attributes: vec![("id".to_string(), "1".to_string())],
        text_content: "hello".to_string(),
    };

    let (sender, rx) = ElementSnapshotSender::new(initial);
    let node = Arc::new(YrsElementNode::new(rx));

    let context = Context::new();
    let mut stream = Node::get_stream(node.clone(), context);

    // First emission: initial value
    let first = stream.next().await.unwrap();
    assert_eq!(first.len(), 1);
    let text = first[0]
        .final_result()
        .as_string_value()
        .unwrap()
        .to_string();
    assert!(text.contains("item"));
    assert!(text.contains("id=\"1\""));
    assert!(text.contains("hello"));

    // Update the snapshot
    sender.send(ElementSnapshot {
        local_name: "item".to_string(),
        attributes: vec![("id".to_string(), "1".to_string())],
        text_content: "updated".to_string(),
    });

    let second = stream.next().await.unwrap();
    let text = second[0]
        .final_result()
        .as_string_value()
        .unwrap()
        .to_string();
    assert!(text.contains("updated"));
}

#[tokio::test]
async fn test_yrs_element_node_current_snapshot() {
    use hick_grove::node_bridge::{ElementSnapshot, ElementSnapshotSender, YrsElementNode};

    let initial = ElementSnapshot {
        local_name: "test".to_string(),
        attributes: vec![],
        text_content: "".to_string(),
    };

    let (_sender, rx) = ElementSnapshotSender::new(initial);
    let node = YrsElementNode::new(rx);

    let snap = node.current_snapshot();
    assert_eq!(snap.local_name, "test");
}

// -----------------------------------------------------------------------
// Phase 7: ElementEvent::Removed + Reactive node bridge
// -----------------------------------------------------------------------

#[tokio::test]
async fn test_yrs_bridge_fires_removed_event() {
    use hick_grove::yrs_bridge::{ElementCache, YrsBridge};
    use tokio::sync::mpsc;
    use yrs::{Doc, Transact, WriteTxn, Xml, XmlFragment, XmlFragmentRef};

    let doc = Doc::new();
    let mut ns = HashMap::new();
    ns.insert("task".to_string(), NamespaceUri::new(TASK_NS));

    let (tx, mut rx) = mpsc::unbounded_channel();
    let _bridge = YrsBridge::attach(&doc, ns, tx, ElementCache::new());

    // Insert an element
    {
        let mut txn = doc.transact_mut();
        let root: XmlFragmentRef = txn.get_or_insert_xml_fragment("root");
        let elem = root.insert(&mut txn, 0, yrs::XmlElementPrelim::empty("task:create"));
        elem.insert_attribute(&mut txn, "id", "task-1");
        elem.insert_attribute(&mut txn, "description", "Buy groceries");
    }

    // Drain creation events
    while rx.try_recv().is_ok() {}

    // Now remove the element
    {
        let mut txn = doc.transact_mut();
        let root: XmlFragmentRef = txn.get_or_insert_xml_fragment("root");
        root.remove_range(&mut txn, 0, 1);
    }

    // Should have a Removed event
    let mut events = Vec::new();
    while let Ok(ev) = rx.try_recv() {
        events.push(ev);
    }

    let removed = events.iter().find(|e| {
        matches!(
            &e.event,
            ElementEvent::Removed { local_name, .. } if local_name == "create"
        )
    });
    assert!(
        removed.is_some(),
        "Should have received a Removed event; got: {:?}",
        events
    );
    assert_eq!(removed.unwrap().namespace_uri.as_str(), TASK_NS);

    if let ElementEvent::Removed {
        local_name,
        attributes,
        element_id,
    } = &removed.unwrap().event
    {
        assert_eq!(local_name, "create");
        assert!(attributes.iter().any(|(k, v)| k == "id" && v == "task-1"));
        assert!(
            attributes
                .iter()
                .any(|(k, v)| k == "description" && v == "Buy groceries")
        );
        assert_eq!(element_id.as_deref(), Some("task-1"));
    }
}

#[tokio::test]
async fn test_engine_removal_dispatches_to_handler() {
    let handler = Arc::new(RecordingHandler::new(TASK_NS));
    let handler_clone = handler.clone();

    let mut registry = NamespaceRegistry::new();
    registry.register(handler);

    let engine = GroveEngine::new(GroveConfig { registry });

    let doc_id = engine
        .create_document(vec![("task".to_string(), NamespaceUri::new(TASK_NS))])
        .await;

    // First, create an element
    let xml = r#"<message xmlns:task="https://example.com/vocab/task#">
        <task:create id="task-1" description="Buy milk">created a task</task:create>
    </message>"#;

    engine
        .process_command(GroveCommand::StructuredMessage {
            doc_id: doc_id.clone(),
            xml: xml.to_string(),
        })
        .await
        .unwrap();

    // Verify Created event
    let events = handler_clone.recorded_events();
    assert!(
        events.iter().any(
            |e| matches!(e, ElementEvent::Created { local_name, .. } if local_name == "create")
        )
    );

    // Now remove the element directly via yrs
    let meta = engine.doc_store().get(&doc_id).unwrap();
    {
        use yrs::{Transact, WriteTxn, XmlFragment};

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let _bridge = hick_grove::yrs_bridge::YrsBridge::attach(
            &meta.doc,
            meta.namespaces.clone(),
            tx,
            meta.element_cache.clone(),
        );

        {
            let mut txn = meta.doc.transact_mut();
            let root: yrs::XmlFragmentRef = txn.get_or_insert_xml_fragment("root");
            // The root has: message element -> task:create element
            // Remove the message element (index 0 of root)
            root.remove_range(&mut txn, 0, 1);
        }

        let mut dispatcher =
            hick_grove::dispatcher::Dispatcher::new(engine.registry().clone(), rx, doc_id.clone());
        dispatcher.drain_pending();
    }

    // Handler should now have both Created and Removed
    let events = handler_clone.recorded_events();
    let removed = events
        .iter()
        .find(|e| matches!(e, ElementEvent::Removed { .. }));
    assert!(
        removed.is_some(),
        "Handler should have received a Removed event; all events: {:?}",
        events
    );
}

#[tokio::test]
async fn test_reactive_handler_create_update_remove() {
    use hick_grove::namespace::NamespaceUri;
    use hick_grove::node_bridge::ElementIdentity;
    use hick_grove::plugin::{ElementEvent, HandlerContext, NamespaceHandler};
    use hick_grove::reactive_handler::ReactiveNamespaceHandler;

    let ns = NamespaceUri::new(TASK_NS);
    let handler = ReactiveNamespaceHandler::new(ns.clone());
    let ctx = HandlerContext {
        doc_id: hick_grove::namespace::DocId::from_string("doc-test"),
    };

    let identity = ElementIdentity {
        namespace_uri: ns.clone(),
        local_name: "create".to_string(),
        id: "task-1".to_string(),
    };

    let node = handler.subscribe(identity.clone());
    assert!(handler.is_subscribed(&identity));

    // Initial snapshot is empty
    let snap = node.current_snapshot();
    assert_eq!(snap.local_name, "create");
    assert!(snap.attributes.is_empty());

    // Send Created event
    handler
        .handle_event(
            &ElementEvent::Created {
                local_name: "create".to_string(),
                attributes: vec![
                    ("id".to_string(), "task-1".to_string()),
                    ("description".to_string(), "Buy milk".to_string()),
                ],
                text_content: "created a task".to_string(),
                element_id: Some("task-1".to_string()),
            },
            &ctx,
        )
        .unwrap();

    let snap = node.current_snapshot();
    assert_eq!(snap.attributes.len(), 2);
    assert_eq!(snap.text_content, "created a task");

    // Send AttributeChanged event
    handler
        .handle_event(
            &ElementEvent::AttributeChanged {
                local_name: "create".to_string(),
                attr_name: "description".to_string(),
                old: Some("Buy milk".to_string()),
                new: Some("Buy eggs".to_string()),
                element_id: Some("task-1".to_string()),
            },
            &ctx,
        )
        .unwrap();

    let snap = node.current_snapshot();
    let desc = snap
        .attributes
        .iter()
        .find(|(k, _)| k == "description")
        .unwrap();
    assert_eq!(desc.1, "Buy eggs");

    // Send TextChanged event
    handler
        .handle_event(
            &ElementEvent::TextChanged {
                local_name: "create".to_string(),
                new_text: "updated task".to_string(),
                element_id: Some("task-1".to_string()),
            },
            &ctx,
        )
        .unwrap();

    let snap = node.current_snapshot();
    assert_eq!(snap.text_content, "updated task");

    // Send Removed event — should drop the subscription
    handler
        .handle_event(
            &ElementEvent::Removed {
                local_name: "create".to_string(),
                attributes: vec![],
                element_id: Some("task-1".to_string()),
            },
            &ctx,
        )
        .unwrap();

    assert!(!handler.is_subscribed(&identity));
}

#[tokio::test]
async fn test_reactive_handler_identity_isolation() {
    use hick_grove::namespace::NamespaceUri;
    use hick_grove::node_bridge::ElementIdentity;
    use hick_grove::plugin::{ElementEvent, HandlerContext, NamespaceHandler};
    use hick_grove::reactive_handler::ReactiveNamespaceHandler;

    let ns = NamespaceUri::new(TASK_NS);
    let handler = ReactiveNamespaceHandler::new(ns.clone());
    let ctx = HandlerContext {
        doc_id: hick_grove::namespace::DocId::from_string("doc-test"),
    };

    let id1 = ElementIdentity {
        namespace_uri: ns.clone(),
        local_name: "create".to_string(),
        id: "task-1".to_string(),
    };
    let id2 = ElementIdentity {
        namespace_uri: ns.clone(),
        local_name: "create".to_string(),
        id: "task-2".to_string(),
    };

    let node1 = handler.subscribe(id1.clone());
    let _node2 = handler.subscribe(id2.clone());

    // Fire Created for task-2 only
    handler
        .handle_event(
            &ElementEvent::Created {
                local_name: "create".to_string(),
                attributes: vec![
                    ("id".to_string(), "task-2".to_string()),
                    ("description".to_string(), "Task two".to_string()),
                ],
                text_content: "".to_string(),
                element_id: Some("task-2".to_string()),
            },
            &ctx,
        )
        .unwrap();

    // node1 should still have empty attributes (no event matched task-1)
    let snap1 = node1.current_snapshot();
    assert!(
        snap1.attributes.is_empty(),
        "task-1 node should not have received task-2's event"
    );

    // Fire Created for task-1
    handler
        .handle_event(
            &ElementEvent::Created {
                local_name: "create".to_string(),
                attributes: vec![
                    ("id".to_string(), "task-1".to_string()),
                    ("description".to_string(), "Task one".to_string()),
                ],
                text_content: "".to_string(),
                element_id: Some("task-1".to_string()),
            },
            &ctx,
        )
        .unwrap();

    let snap1 = node1.current_snapshot();
    assert_eq!(snap1.attributes.len(), 2);
    let desc = snap1
        .attributes
        .iter()
        .find(|(k, _)| k == "description")
        .unwrap();
    assert_eq!(desc.1, "Task one");
}

// -----------------------------------------------------------------------
// State vector / diff
// -----------------------------------------------------------------------

#[tokio::test]
async fn test_state_vector_and_diff() {
    let registry = NamespaceRegistry::new();
    let engine = GroveEngine::new(GroveConfig { registry });

    let doc_id = engine
        .create_document(vec![("task".to_string(), NamespaceUri::new(TASK_NS))])
        .await;

    let sv = engine.state_vector(&doc_id).unwrap();
    assert!(!sv.is_empty());

    // Diff from empty state vector should include all data
    use yrs::updates::encoder::Encode;
    let empty_sv = yrs::StateVector::default().encode_v1();
    let diff = engine.encode_diff(&doc_id, &empty_sv).unwrap();
    assert!(!diff.is_empty());
}
