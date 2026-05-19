# hick-merge

Three-way merge system for reconciling pipeline-generated file output with user
edits.

When the pipeline regenerates a file that the user has also edited, `hick-merge`
uses the stored base snapshot (from `hick-store`) to perform a three-way merge,
calling a conflict resolution strategy only when there are genuine conflicts.

## Merge strategies

| Strategy | Behaviour |
|---|---|
| `FailOnConflict` | Returns an error on any conflict |
| `KeepEdited` | Always preserves the user's version |
| `TakeGenerated` | Always accepts the new pipeline output |
| `LlmMergeStrategy` | Delegates conflict resolution to an LLM API with optional caching |

`MergeOrchestrator` orchestrates the three-way merge using a `VersionStore`
snapshot as the base. `MergeResult` carries the resolved content and any
conflict metadata.
