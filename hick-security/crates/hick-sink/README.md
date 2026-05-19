# hick-sink

API sink abstraction that controls which external services a container may call
and what data may flow into them.

`SinkDef` declares a sink type with typed `SinkSlot`s that carry
accepted/rejected `Classification` lists. `SinkValidator::validate_call` enforces
the full rule set — token API-sink permission, expiry, max-calls, recipient
constraints, and per-slot classification policies — against a
`ContainerCapabilities` from `hick-token`.

## Key types

- `SinkDef` — declaration of a sink and its typed input slots
- `SinkSlot` — a named slot with classification allow/deny lists
- `SinkValidator` — enforces capability rules before any outbound call
- `SinkBackend` — async execution trait for the actual call
- `MockSinkBackend` — records calls for test assertions
