---
id: TASK-14
title: Create hick-agent-hick-edit extension crate (hick XML edit format)
status: To Do
assignee: []
created_date: '2026-05-15 12:44'
labels: []
milestone: m-0
dependencies:
  - TASK-12
references:
  - hick-agent/crates/hick-agent-core/src/lib.rs
  - hick-agent/crates/hick-agent-core/src/edit_format.rs
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Consolidate all hick-XML-aware agent behavior into a single `hick-agent-hick-edit` extension crate. This is the crate that knows about `hick:file`, `hick:copy`, `hick:paste`, and the pipeline structure. It is currently spread across two wrongly-named crates.

**What moves here:**

From `hick-agent-core` (currently misnamed — it's actually hick-specific):
- `HickEditFormat` — the edit format parser/emitter
- `HickEditService` — orchestrates edit format operations against the live pipeline
- `PipelineOutput` — represents the rendered output of a pipeline step
- `QueryMatch` / `SourceEdit` — edit location types

From `hick-agent-engine`:
- `PipelineWatcher` — watches `_hick.yml` and reruns `hick run` on change
- Any reverse-provenance logic (mapping output file positions back to hick:file source locations)

**New types to add:**
- `HickEditAgent` — a composable behavior struct that, given an `AgentRuntime` from the sdk, adds hick-edit capabilities: discovers paste slots in the pipeline, produces `hick:copy` elements for modifications, produces `hick:file` elements for new files
- `PasteSlotRegistry` — indexes the named paste slots in the current pipeline so the agent can discover composition points without reading every file

**Dependency graph:**
```
hick-agent-hick-edit → hick-agent-sdk
hick-agent-hick-edit → hick (pipeline execution)
hick-agent-hick-edit → hick-lang (XML parsing)
```

**Note on hick-agent-core:** After this task, `hick-agent-core` should be empty of hick-specific logic. Either delete the crate or repurpose it as an alias. The naming inversion (core = hick-specific, engine = framework) that exists today should be fully resolved by TASK-12 + TASK-13 + TASK-14 together.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 hick-agent-hick-edit crate exists at hick-agent/crates/hick-agent-hick-edit/
- [ ] #2 HickEditFormat, HickEditService, PipelineOutput, QueryMatch, SourceEdit are pub use'd with doc comments
- [ ] #3 PipelineWatcher is pub use'd with doc comments
- [ ] #4 hick-agent-core is either deleted or contains only re-exports pointing to their new homes (no logic)
- [ ] #5 HickEditAgent struct composes with AgentRuntime from hick-agent-sdk
- [ ] #6 cargo check --package hick-agent-hick-edit passes
- [ ] #7 cargo test --workspace passes
<!-- AC:END -->
