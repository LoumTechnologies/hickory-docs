<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-grove/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `f886946b82db4d1c967241fdee017bf283bd77331ab693291285f86925f324f1`.

<book:entry path="crates/hick-grove/README.md">
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
</book:entry>

</book:doc>
