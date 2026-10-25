<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-merge/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `d30c1c2de107ba383aec0ecb39b32c23c1a08beed16753a5ca19b2ea8a52c357`.

<book:entry path="crates/hick-merge/README.md">
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
</book:entry>

</book:doc>
