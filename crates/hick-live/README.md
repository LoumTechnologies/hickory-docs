# hick-live

Reactive streaming data layer for hick pipelines, porting the LiveLinq pattern
to Rust over Apache Arrow `RecordBatch` streams.

## Operators

Set-based `ChangeStream` transformers: `Where`, `Select`, `GroupBy`, `SelectMany`.

## Key types

- `RecordChange` — an insert/update/delete event on a record batch
- `ChangeSource` — a streaming data source
- `LiveSchema` — schema descriptor
- `RecordChangeNode` / `StreamingStructuredDataNode` — wrappers that integrate
  these streams into the `hick-exec` reactive node graph

Enables pipeline steps that produce or consume structured, incrementally updated
tabular data — think live query results that flow through the document generation
graph as records arrive.
