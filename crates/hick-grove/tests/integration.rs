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
    use hick_grove::yrs_bridge::YrsBridge;
    use tokio::sync::mpsc;
    use yrs::{Doc, Text, Transact, WriteTxn, Xml, XmlFragment, XmlFragmentRef};

    let doc = Doc::new();
    let mut ns = HashMap::new();
    ns.insert("task".to_string(), NamespaceUri::new(TASK_NS));

    let (tx, mut rx) = mpsc::unbounded_channel();
    let _bridge = YrsBridge::attach(&doc, ns, tx);

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
