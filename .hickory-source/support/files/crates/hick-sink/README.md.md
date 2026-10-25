<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-sink/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `39f538050d220bbc5d7968c6a7d671ee1e91088bee342996fe7061f1eebb9260`.

<book:entry path="crates/hick-sink/README.md">
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
</book:entry>

</book:doc>
