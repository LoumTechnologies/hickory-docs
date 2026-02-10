# Grove: Reactive XML Document Engine

Grove is an embeddable engine that turns multi-namespace XML messages into
CRDT-backed documents with event-driven projections. It implements the
structured messages pattern described in `structured_messages.md`: human-
readable XML carries semantic annotations, and namespace handlers project
those annotations into external stores (SQLite, search indices, etc.).

---

## How It Works

The pipeline has four stages:

1. **Parse** -- A structured XML message is parsed into a tree of
   namespace-qualified nodes.
2. **Apply** -- The parsed tree is inserted into a Yrs CRDT document.
3. **Bridge** -- `observe_deep` on the Yrs document emits fine-grained
   change events tagged with their namespace URI.
4. **Dispatch** -- Events are routed to registered `NamespaceHandler`
   implementations, which produce side effects (DB writes, logs, etc.).

```text
XML message --> MessageParser --> Yrs Doc (CRDT)
                                      |
                                 YrsBridge (observe_deep)
                                      |
                                 Dispatcher --> NamespaceHandler(s)
                                                     |
                                                Side effects
```

Because the document is a Yrs CRDT, multiple clients can concurrently edit
the same document and changes merge automatically. The binary sync protocol
(state vectors and diffs) is exposed directly on the engine.

---

## Creating a Document

Documents live inside a `GroveEngine`. To create one, pass the namespace
prefixes that messages will use:

```rust,ignore
use std::sync::Arc;
use hick_grove::{GroveConfig, GroveEngine};
use hick_grove::namespace::NamespaceUri;
use hick_grove::plugin::NamespaceRegistry;

let registry = NamespaceRegistry::new();
let engine = GroveEngine::new(GroveConfig { registry });

let doc_id = engine.create_document(vec![
    ("task".into(), NamespaceUri::new("https://example.com/vocab/task#")),
    ("user".into(), NamespaceUri::new("https://example.com/vocab/user#")),
]).await;
```

The returned `DocId` is a UUID v7 string (e.g. `doc-0192a3b4...`) that
identifies the document in all subsequent operations.

---

## Sending a Structured Message

A structured message is XML where namespace-prefixed tags carry semantic
meaning. The text between tags is human-readable prose:

```xml
<message xmlns:task="https://example.com/vocab/task#"
         xmlns:user="https://example.com/vocab/user#">
  Hey <user:mention ref="joe-1">Joe</user:mention>,
  <task:create id="task-1" description="Change oil in the E-350">
    created and <task:assign to="joe-1" ref="task-1">assigned this to you</task:assign>.
  </task:create>
  Can you get it done by <task:due date="2025-01-15" ref="task-1">tomorrow COB</task:due>?
</message>
```

Send it to the engine:

```rust,ignore
use hick_grove::command::GroveCommand;

let result = engine.process_command(GroveCommand::StructuredMessage {
    doc_id: doc_id.clone(),
    xml: message_xml.to_string(),
}).await?;

println!("{} events dispatched", result.events_dispatched);
```

The engine parses the XML, applies it to the Yrs document, and dispatches
events to every handler registered for the namespaces used in the message.

---

## The Namespace Handler Plugin System

Handlers are the extension point. Each handler is registered for a namespace
URI and receives `ElementEvent`s when elements in that namespace are created,
changed, or removed.

### The `NamespaceHandler` trait

```rust,ignore
pub trait NamespaceHandler: Send + Sync {
    /// The namespace URI this handler responds to.
    fn namespace_uri(&self) -> &NamespaceUri;

    /// A human-readable name for logging.
    fn name(&self) -> &str;

    /// Handle a single element event, returning zero or more side effects.
    fn handle_event(
        &self,
        event: &ElementEvent,
        ctx: &HandlerContext,
    ) -> anyhow::Result<Vec<SideEffect>>;
}
```

### Element events

| Variant | Fired when |
|---------|------------|
| `Created { local_name, attributes, text_content }` | A new element appears in the document |
| `AttributeChanged { local_name, attr_name, old, new }` | An attribute is added, changed, or removed |
| `TextChanged { local_name, new_text }` | The text content of an element changes |
| `Removed { local_name, attributes }` | An element is removed from the document |

### Registering handlers

```rust,ignore
use hick_grove::plugin::NamespaceRegistry;

let mut registry = NamespaceRegistry::new();
registry.register(Arc::new(my_task_handler));
registry.register(Arc::new(my_user_handler));

let engine = GroveEngine::new(GroveConfig { registry });
```

Multiple handlers can be registered for the same namespace URI. They are all
called for every event in that namespace.

---

## Writing a Custom NamespaceHandler

Here is a minimal handler that logs every element creation in a custom
namespace:

```rust,ignore
use hick_grove::namespace::NamespaceUri;
use hick_grove::plugin::{ElementEvent, HandlerContext, NamespaceHandler, SideEffect};

pub struct AuditHandler {
    namespace: NamespaceUri,
}

impl AuditHandler {
    pub fn new() -> Self {
        Self {
            namespace: NamespaceUri::new("https://example.com/vocab/audit#"),
        }
    }
}

impl NamespaceHandler for AuditHandler {
    fn namespace_uri(&self) -> &NamespaceUri {
        &self.namespace
    }

    fn name(&self) -> &str {
        "audit"
    }

    fn handle_event(
        &self,
        event: &ElementEvent,
        ctx: &HandlerContext,
    ) -> anyhow::Result<Vec<SideEffect>> {
        match event {
            ElementEvent::Created { local_name, attributes, .. } => {
                Ok(vec![SideEffect::Log(
                    format!("[{}] created <{}> with {:?}", ctx.doc_id, local_name, attributes),
                )])
            }
            _ => Ok(vec![]),
        }
    }
}
```

---

## The Tasks Example Plugin

The crate ships with an example handler behind the `sqlite-example` feature
flag. It projects task-related events into an in-memory SQLite database.

### Namespace

`https://example.com/vocab/task#`

### Supported elements

| Element | Attributes | Effect |
|---------|------------|--------|
| `<task:create>` | `id`, `description` | Inserts a row into the `tasks` table |
| `<task:assign>` | `ref` (or `task`), `to` | Sets the `assignee` column |
| `<task:due>` | `ref` (or `task`), `date` | Sets the `due_date` column |

### Usage

```rust,ignore
use std::sync::Arc;
use hick_grove::plugins::tasks::{TasksHandler, TasksSqliteStore};

let store = Arc::new(TasksSqliteStore::new_in_memory()?);
let handler = Arc::new(TasksHandler::new(store.clone()));

let mut registry = NamespaceRegistry::new();
registry.register(handler);

// After processing messages, query the read model:
let tasks = store.list_tasks(doc_id.as_str())?;
for task in &tasks {
    println!("{}: {} (assigned to {:?})", task.id, task.description, task.assignee);
}
```

---

## hick-flow Integration via YrsElementNode

`YrsElementNode` bridges a Yrs element into hick-flow's reactive `Node`
system. Each time the element changes in the CRDT, a new `ElementSnapshot`
is pushed through a `tokio::sync::watch` channel, and `get_stream()` yields
the updated value as `NodeTrace` batches.

This lets you compose Yrs-backed elements with `InsertionPoint`,
`DynamicCombineLatest`, and other hick-flow primitives.

```rust,ignore
use hick_grove::node_bridge::{ElementSnapshot, ElementSnapshotSender, YrsElementNode};
use hick_flow::Context;

let initial = ElementSnapshot {
    local_name: "status".into(),
    attributes: vec![("value".into(), "open".into())],
    text_content: String::new(),
};

let (sender, rx) = ElementSnapshotSender::new(initial);
let node = Arc::new(YrsElementNode::new(rx));

// Use node in a hick-flow graph
let stream = node.get_stream(Context::empty());

// When the Yrs element changes, push a new snapshot:
sender.send(ElementSnapshot {
    local_name: "status".into(),
    attributes: vec![("value".into(), "closed".into())],
    text_content: String::new(),
});
```

---

## Binary Sync with Yrs

The engine exposes Yrs state vectors and diffs for peer-to-peer or
client-server sync:

```rust,ignore
// Get this document's state vector (binary, Yrs v1 encoding)
let sv = engine.state_vector(&doc_id)?;

// Given a remote peer's state vector, compute the diff they need
let diff = engine.encode_diff(&doc_id, &remote_sv)?;

// Apply a binary update from a remote peer
engine.process_command(GroveCommand::BinaryUpdate {
    doc_id: doc_id.clone(),
    update: remote_diff,
}).await?;
```

The `hick-grove-server` crate wraps these operations in HTTP endpoints and a
WebSocket handler for real-time sync. See [Grove Reference](grove-reference.md)
for the full server API.

---

## Running the Server

The `hick-grove-server` binary starts an Axum HTTP server with the tasks
example plugin pre-configured:

```bash
cd crates/hick-grove-server
cargo run
# Listening on 0.0.0.0:3000
```

### Quick smoke test

```bash
# Create a document
DOC=$(curl -s -X POST http://localhost:3000/documents \
  -H 'Content-Type: application/json' \
  -d '{"namespaces":{"task":"https://example.com/vocab/task#"}}' \
  | jq -r .id)

# Send a structured message
curl -s -X POST "http://localhost:3000/documents/$DOC/messages" \
  -H 'Content-Type: application/json' \
  -d "{\"xml\":\"<msg xmlns:task=\\\"https://example.com/vocab/task#\\\"><task:create id=\\\"t-1\\\" description=\\\"Buy milk\\\"/></msg>\"}"
```

See [Grove Reference](grove-reference.md) for the complete endpoint list.
