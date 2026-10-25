<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-exec/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `3be873d7cb4e2afb8409a79f2bd83ff3c7c62ae2a3ec71d9ea2a38688af7ff14`.

<book:entry path="crates/hick-exec/README.md">
# hick-exec

Reactive execution engine for `.md` pipelines.

Implements a port of `DynamicCombineLatest`, `Node`, `InsertionPoint`, and
related reactive types, and adds an information-flow DAG for validating execution
order and detecting cycles.

## Key abstractions

- `Node` — object-safe, async streaming trait; the fundamental unit of the pipeline graph
- `InsertionPoint` — ordered child collection that combines child streams
- `StringNode` / `SeparatorNode` / `TransformNode` — leaf and composite node types
- `converge_to_string` — terminal consumer that collects a node graph into a `String`
- `MultiDocumentState` — global mutable pipeline state (files, copy/paste blocks, variables)
- `FlowDag` / `DagEdge` / `ExecId` — cycle-detection DAG for execution order validation
</book:entry>

</book:doc>
