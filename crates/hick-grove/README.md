# hick-grove

Embeddable reactive XML document engine built on the Yrs CRDT library.

`GroveEngine` accepts `GroveCommand` messages (either parsed XML or raw Yrs
binary updates), applies them to a per-document Yrs `XmlFragment` inside a
transaction, and dispatches resulting change events to namespace-specific
`NamespaceHandler` implementations via a `Dispatcher`.

## Key types

- `GroveEngine` — central coordinator, manages per-document Yrs documents
- `GroveCommand` — input: parsed XML or raw Yrs binary update
- `NamespaceHandler` — plugin trait for handling elements in a specific XML namespace
- `YrsBridge` — observes deep changes on a Yrs document and forwards typed events
- `YrsElementNode` — bridges a Yrs element into the `hick-flow` reactive `Node`
  system via `tokio::sync::watch`

A built-in `plugins::tasks::TasksHandler` (with optional SQLite backend) is
included as a reference plugin implementation.
