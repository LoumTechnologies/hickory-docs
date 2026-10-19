# Runtime observations

`hick-observe` is the Rust-owned foundation for issue #54. It stores version 1
JSON run records under `<project>/.hick/runs/`, independently of transcript cache
keys, Git history, agent refresh policy and authored Markdown. It does not yet
connect language adapters or expose UI/API routes (#55–#58, #64–#65).

A host captures exact source bytes with `Source::capture`, creates a `Run`, and
saves it before execution. Implement `ObservationProvider` to return actual
runtime evidence. Call `Run::collect` and `LocalStore::save` periodically; call
`finish` with the actual terminal status and save again. Provider errors leave
already received events available for saving; the host chooses failed or partial.
`cargo run -p hick-observe --example local_run` exercises this path in a temporary
project. Providers should use the existing DAP `Mapping`/literate position maps
when a runtime supplies a location; no second mapper is introduced here.

Run UUIDs are occurrence identities, never cache keys. Event IDs must be unique
within a run; the durable handle is `(run.id, event.id)`. Source IDs name captured
bytes within that run; revision is their SHA-256. Block IDs are provider supplied,
not derived from shifting line numbers. Invocation/parent/thread/task identities
are optional provider facts. Receive order does not establish execution causality.
Emission and caller locations remain separate. Byte ranges are zero based and
half open in captured source; adapters must convert their native coordinates.
A profile retains provider, mapping, clock domain, units, interval and aggregation
scope. Reproducible source attribution makes no performance timing promise.

`output_lines` groups only the requested stream/category and returns raw bytes
plus each contributing event's byte ranges. LF ends a line; CR and escape bytes
are retained literally. Consumers must escape terminal controls when rendering
and decode UTF-8 after assembly. This is lossless framing, not terminal emulation.
Stderr stays output unless the provider explicitly supplies an exception.

The store holds an exclusive writer lease. Reopening after interruption converts
saved running records to partial. Unsaved events are unrecoverable. Each save is
an atomic file replacement with file and directory sync; an incomplete temporary
file is ignored. Captured sources and the saved event prefix cannot be rewritten.
A missing file returns `Missing`; corrupt/unsupported records return `Corrupt`
and are never silently overwritten or evicted. Auxiliary unavailable evidence
belongs in `missing_artifacts`; all captured source bytes are embedded in JSON.

Hosts invoke `retain(Retention::default(), now_unix_ms)` after finishing runs:
keep at most 100 unpinned terminal runs, 256 MiB of serialized unpinned records,
and 30 days from run start. Oldest goes first. Pins and active runs are exempt;
corrupt files need explicit inspection/deletion. Explicit `delete` can remove a
pin. Pinning does not export to Markdown. `collect` limits total serialized event
bytes; overflow marks truncation and stops draining. Source size is host bounded;
`append` is the low-level per-event limit. Retention is an explicit host operation,
not a background service. Pins, active runs and source snapshots can exceed the
budget. No unlimited Git commits are created. Cache clear and existing explicit
`hick:output`/`hick:ingested` behavior are untouched.
