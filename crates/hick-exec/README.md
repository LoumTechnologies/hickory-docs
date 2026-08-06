# hick-exec

Reactive execution engine for `.hick` pipelines.

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
