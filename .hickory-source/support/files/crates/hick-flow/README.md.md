<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-flow/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `9e6a6c838d2b8acb26427ded514d2c86dc63552b69f7114a259287478065879c`.

<book:entry path="crates/hick-flow/README.md">
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
back to their `.md` source file positions or container/script origins.

Terminal consumers call `converge` / `converge_to_string` to collect the tree
into a final string with optional provenance attached.
</book:entry>

</book:doc>
