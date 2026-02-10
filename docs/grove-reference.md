# Grove Reference

hick-grove is an embeddable reactive XML document engine. It implements the
structured messages pattern: multi-namespace XML is parsed, applied to a Yrs
CRDT document, and change events are dispatched to namespace-specific handlers.

hick-grove-server is a thin Axum HTTP/WebSocket binary that exposes the engine
over the network.

---

## Table of Contents

1. [GroveEngine](#1-groveengine)
2. [GroveCommand](#2-grovecommand)
3. [CommandResult](#3-commandresult)
4. [NamespaceHandler Trait](#4-namespacehandler-trait)
5. [ElementEvent](#5-elementevent)
6. [SideEffect](#6-sideeffect)
7. [HandlerContext](#7-handlercontext)
8. [NamespaceRegistry](#8-namespaceregistry)
9. [Message Parser](#9-message-parser)
10. [Namespace Types](#10-namespace-types)
11. [Node Bridge](#11-node-bridge)
12. [Yrs Bridge](#12-yrs-bridge)
13. [DocStore](#13-docstore)
14. [Dispatcher](#14-dispatcher)
15. [Server Endpoints](#15-server-endpoints)
16. [WebSocket Protocol](#16-websocket-protocol)

---

## 1. GroveEngine

**Module:** `hick_grove`

The top-level entry point. Owns the document store and handler registry.

### Construction

```rust,ignore
use hick_grove::{GroveConfig, GroveEngine};

let engine = GroveEngine::new(GroveConfig { registry });
```

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `(config: GroveConfig) -> Self` | Create an engine with the given registry |
| `create_document` | `(&self, namespaces: Vec<(String, NamespaceUri)>) -> DocId` | Create a new document with namespace declarations. Returns a `DocId` (UUID v7) |
| `process_command` | `(&self, command: GroveCommand) -> Result<CommandResult>` | Process a structured message or binary update |
| `state_vector` | `(&self, doc_id: &DocId) -> Result<Vec<u8>>` | Get the Yrs state vector (v1 encoding) for sync |
| `encode_diff` | `(&self, doc_id: &DocId, remote_sv: &[u8]) -> Result<Vec<u8>>` | Compute the diff a remote peer needs given their state vector |
| `doc_store` | `(&self) -> &Arc<DocStore>` | Access the underlying document store |
| `registry` | `(&self) -> &Arc<NamespaceRegistry>` | Access the handler registry |

### GroveConfig

| Field | Type | Description |
|-------|------|-------------|
| `registry` | `NamespaceRegistry` | The handler registry consumed by the engine |

---

## 2. GroveCommand

**Module:** `hick_grove::command`

Commands that can be applied to the engine.

| Variant | Fields | Description |
|---------|--------|-------------|
| `StructuredMessage` | `doc_id: DocId, xml: String` | Parse and apply an XML message to a document |
| `BinaryUpdate` | `doc_id: DocId, update: Vec<u8>` | Apply a raw Yrs binary update |

---

## 3. CommandResult

**Module:** `hick_grove::command`

Returned after processing a command.

| Field | Type | Description |
|-------|------|-------------|
| `doc_id` | `DocId` | The document that was modified |
| `events_dispatched` | `usize` | Number of events dispatched to handlers |

---

## 4. NamespaceHandler Trait

**Module:** `hick_grove::plugin`

The extension point for domain logic. Handlers receive element events for a
specific namespace URI and produce side effects.

```rust,ignore
pub trait NamespaceHandler: Send + Sync {
    fn namespace_uri(&self) -> &NamespaceUri;
    fn name(&self) -> &str;
    fn handle_event(
        &self,
        event: &ElementEvent,
        ctx: &HandlerContext,
    ) -> anyhow::Result<Vec<SideEffect>>;
}
```

| Method | Description |
|--------|-------------|
| `namespace_uri()` | The namespace URI this handler responds to |
| `name()` | Human-readable name for logging and diagnostics |
| `handle_event()` | Process a single event, returning zero or more side effects |

---

## 5. ElementEvent

**Module:** `hick_grove::plugin`

Describes a change to a namespace-qualified element. Derived from Yrs
`observe_deep` events after namespace resolution.

| Variant | Fields | Fired when |
|---------|--------|------------|
| `Created` | `local_name: String, attributes: Vec<(String, String)>, text_content: String` | A new element appears |
| `AttributeChanged` | `local_name: String, attr_name: String, old: Option<String>, new: Option<String>` | An attribute is inserted, updated, or removed |
| `TextChanged` | `local_name: String, new_text: String` | Text content of an element changes |
| `Removed` | `local_name: String, attributes: Vec<(String, String)>` | An element is removed |

### Attribute semantics

- `old: None, new: Some(v)` -- attribute inserted
- `old: Some(v1), new: Some(v2)` -- attribute updated
- `old: Some(v), new: None` -- attribute removed

---

## 6. SideEffect

**Module:** `hick_grove::plugin`

Side effects produced by handlers.

| Variant | Field | Description |
|---------|-------|-------------|
| `Log` | `String` | A log message |
| `Custom` | `String` | Application-defined side effect |

---

## 7. HandlerContext

**Module:** `hick_grove::plugin`

Context passed to handlers alongside an event.

| Field | Type | Description |
|-------|------|-------------|
| `doc_id` | `DocId` | The document that triggered the event |

---

## 8. NamespaceRegistry

**Module:** `hick_grove::plugin`

Registry mapping namespace URIs to handler implementations.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `() -> Self` | Create an empty registry |
| `register` | `(&mut self, handler: Arc<dyn NamespaceHandler>)` | Register a handler. Multiple handlers per namespace are allowed |
| `handlers_for` | `(&self, ns: &NamespaceUri) -> &[Arc<dyn NamespaceHandler>]` | Get all handlers for a namespace |
| `all_namespaces` | `(&self) -> Vec<&NamespaceUri>` | List all registered namespace URIs |

---

## 9. Message Parser

**Module:** `hick_grove::message_parser`

Parses multi-namespace XML into a tree of `MessageNode`s with resolved
namespace URIs.

### `parse_message`

```rust,ignore
pub fn parse_message(xml: &str) -> anyhow::Result<ParsedMessage>
```

Parses XML using quick-xml. Namespace prefixes declared via `xmlns:` are
resolved so each element carries its full namespace URI, not just the prefix.

### ParsedMessage

| Field | Type | Description |
|-------|------|-------------|
| `namespaces` | `HashMap<String, NamespaceUri>` | Top-level namespace declarations (prefix -> URI) |
| `nodes` | `Vec<MessageNode>` | The parsed message tree |

### MessageNode

| Variant | Fields | Description |
|---------|--------|-------------|
| `Text` | `String` | Literal text content |
| `Element` | `namespace_uri: NamespaceUri, local_name: String, attributes: Vec<(String, String)>, children: Vec<MessageNode>` | A namespace-qualified element |

---

## 10. Namespace Types

**Module:** `hick_grove::namespace`

### NamespaceUri

A newtype around `String` representing a namespace URI.

```rust,ignore
let ns = NamespaceUri::new("https://example.com/vocab/task#");
assert_eq!(ns.as_str(), "https://example.com/vocab/task#");
```

Implements `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`, `Display`,
`Serialize`, `Deserialize`.

### QualifiedName

A namespace-qualified element or attribute name.

| Field | Type | Description |
|-------|------|-------------|
| `namespace_uri` | `NamespaceUri` | The namespace |
| `local_name` | `String` | The local name (without prefix) |

Display format: `{namespace_uri}:local_name`

### DocId

Unique identifier for a document within the engine. Generated as UUID v7
with a `doc-` prefix.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `() -> Self` | Generate a new DocId (UUID v7) |
| `from_string` | `(s: impl Into<String>) -> Self` | Create from an existing string |
| `as_str` | `(&self) -> &str` | Borrow the inner string |

Implements `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`, `Display`,
`Default`, `Serialize`, `Deserialize`.

---

## 11. Node Bridge

**Module:** `hick_grove::node_bridge`

Bridges Yrs XML elements into hick-flow's reactive `Node` system.

### ElementSnapshot

A snapshot of a Yrs XML element's state at a point in time.

| Field | Type | Description |
|-------|------|-------------|
| `local_name` | `String` | The element tag name |
| `attributes` | `Vec<(String, String)>` | Attribute key-value pairs |
| `text_content` | `String` | Concatenated text content |

| Method | Description |
|--------|-------------|
| `to_string_repr()` | Render as an XML string (e.g. `<status value="open"/>`) |

### YrsElementNode

Implements `hick_flow::Node`. Backed by a `tokio::sync::watch` receiver.
Each Yrs change pushes a new `ElementSnapshot`, and `get_stream()` yields
`NodeTrace` batches containing the rendered XML string.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `(rx: watch::Receiver<ElementSnapshot>) -> Self` | Create from a watch receiver |
| `current_snapshot` | `(&self) -> ElementSnapshot` | Get the current snapshot without consuming the stream |

### ElementSnapshotSender

The sending half for pushing element snapshots from the Yrs bridge.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `(initial: ElementSnapshot) -> (Self, watch::Receiver<ElementSnapshot>)` | Create a sender/receiver pair |
| `send` | `(&self, snapshot: ElementSnapshot)` | Push a new snapshot |

---

## 12. Yrs Bridge

**Module:** `hick_grove::yrs_bridge`

Registers `observe_deep` on a Yrs document's root `XmlFragment` and pushes
namespace-resolved events into a tokio mpsc channel.

### YrsChangeEvent

| Field | Type | Description |
|-------|------|-------------|
| `namespace_uri` | `NamespaceUri` | The namespace of the affected element |
| `event` | `ElementEvent` | The element event |

### YrsBridge

| Method | Signature | Description |
|--------|-----------|-------------|
| `attach` | `(doc: &Doc, namespace_map: HashMap<String, NamespaceUri>, tx: mpsc::UnboundedSender<YrsChangeEvent>) -> Self` | Attach to a Yrs document. The returned bridge holds the subscription alive |

**Important:** The `yrs::Subscription` held by the bridge is not
`Send + Sync`. Create bridges on-demand per command rather than storing them
in async mutexes.

### Event resolution

The bridge resolves `prefix:localname` tags from the Yrs document using the
namespace map provided at attach time. It recurses into child elements to
find namespace-qualified tags nested under wrapper elements.

Supported Yrs event types:

- `XmlFragment` events -- child additions at the root or under elements
- `XmlElement` attribute changes -- attribute insert/update/remove
- `XmlText` changes -- text content updates within an element

---

## 13. DocStore

**Module:** `hick_grove::doc_store`

Thread-safe store mapping `DocId` to Yrs documents and their metadata.

### DocMeta

| Field | Type | Description |
|-------|------|-------------|
| `doc` | `yrs::Doc` | The Yrs CRDT document |
| `namespaces` | `HashMap<String, NamespaceUri>` | Prefix-to-URI map declared at creation time |

### DocStore

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `() -> Self` | Create an empty store |
| `create` | `(&self, id: DocId, namespaces: HashMap<String, NamespaceUri>) -> Arc<DocMeta>` | Create and store a new document |
| `get` | `(&self, id: &DocId) -> Option<Arc<DocMeta>>` | Look up a document by ID |
| `contains` | `(&self, id: &DocId) -> bool` | Check if a document exists |

The store uses `Mutex<HashMap<DocId, Arc<DocMeta>>>` internally.

---

## 14. Dispatcher

**Module:** `hick_grove::dispatcher`

Dispatches `YrsChangeEvent`s from the bridge to registered namespace handlers.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `(registry: Arc<NamespaceRegistry>, rx: mpsc::UnboundedReceiver<YrsChangeEvent>, doc_id: DocId) -> Self` | Create a dispatcher |
| `run` | `(self) -> Vec<SideEffect>` | Async loop: consume events until the channel closes |
| `drain_pending` | `(&mut self) -> Vec<SideEffect>` | Non-blocking: drain currently available events |

The engine uses `drain_pending` for synchronous command processing (the
bridge subscription fires events during the Yrs transaction, so all events
are already in the channel when the transaction drops).

---

## 15. Server Endpoints

**Crate:** `hick-grove-server`

The server runs on `0.0.0.0:3000` by default and uses the tasks example
plugin.

### POST /documents

Create a new document.

**Request:**

```json
{
  "namespaces": {
    "task": "https://example.com/vocab/task#"
  }
}
```

**Response:**

```json
{
  "id": "doc-0192a3b4..."
}
```

### POST /documents/{id}/messages

Send a structured XML message.

**Request:**

```json
{
  "xml": "<msg xmlns:task=\"https://example.com/vocab/task#\"><task:create id=\"t-1\" description=\"Buy milk\"/></msg>"
}
```

**Response:**

```json
{
  "events_dispatched": 1
}
```

### POST /documents/{id}/updates

Apply a raw Yrs binary update.

**Request:**

```json
{
  "update": "<base64-encoded Yrs update>"
}
```

**Response:**

```json
{
  "events_dispatched": 0
}
```

### GET /documents/{id}/state-vector

Get the Yrs state vector for sync.

**Response:**

```json
{
  "state_vector": "<base64-encoded state vector>"
}
```

### POST /documents/{id}/diff

Compute the diff a remote peer needs.

**Request:**

```json
{
  "remote_state_vector": "<base64-encoded remote state vector>"
}
```

**Response:**

```json
{
  "diff": "<base64-encoded diff>"
}
```

### GET /documents/{id}/ws

Upgrade to WebSocket for real-time sync. See
[WebSocket Protocol](#16-websocket-protocol).

---

## 16. WebSocket Protocol

Connect to `ws://host:3000/documents/{id}/ws` for real-time binary sync.

### Connection flow

1. **Server sends** the current state vector as a binary WebSocket message
   on connect.
2. **Client sends** binary messages containing Yrs updates (v1 encoding).
3. **Server applies** the update, dispatches events to handlers, and sends
   back the new state vector as a binary message.
4. **Close** with a standard WebSocket close frame.

### Message types

| Direction | Type | Content |
|-----------|------|---------|
| Server -> Client | Binary | Yrs state vector (on connect and after each update) |
| Client -> Server | Binary | Yrs binary update (v1 encoding) |
| Either | Close | Standard close frame |

All other message types are ignored.
