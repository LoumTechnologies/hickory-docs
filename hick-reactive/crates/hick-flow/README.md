# hick-flow

Core reactive dataflow primitives that power document generation in the hick
pipeline.

## Node trait

`Node` is an object-safe, `Arc`-backed tree node that yields a
`Stream<Vec<NodeTrace>>`. The stream emits whenever any upstream input changes,
enabling incremental re-generation of only the affected parts of the output.

## Leaf types

- `StringNode` — static string content
- `BinaryNode` — binary content
- `SpanNode` — content with source provenance

## Combinators

- `DynamicCombineLatest` — merges variable-length child streams
- `TransformNode` — applies a function to combined child output
- `InsertionPoint` — ordered child collection with dynamic membership

## Provenance

`SourceOrigin`, `ProvenanceMap`, and `ProvenanceSpan` map generated output bytes
back to their `.hick` source file positions or container/script origins.

Terminal consumers call `converge` / `converge_to_string` to collect the tree
into a final string with optional provenance attached.
