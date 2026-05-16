---
id: TASK-18
title: 'Agent produces hick:copy edits instead of direct file writes'
status: In Progress
assignee: []
created_date: '2026-05-15 12:46'
updated_date: '2026-05-16 11:24'
labels: []
milestone: m-1
dependencies:
  - TASK-14
  - TASK-15
  - TASK-17
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Change the agent's editing behavior so that instead of writing files directly, it produces `hick:copy` elements (for modifications to existing files) and `hick:file` elements (for new files), appending them to the repository's pipeline. Direct file writes are replaced by pipeline edits.

**Why:** Direct file writes bypass the hickory pipeline entirely — the edited file has no provenance, cannot be replayed, and creates merge conflicts with other agents. hick:copy edits are merge-safe (additive composition at paste slots) and the pipeline is the authoritative source of truth.

**Agent behavior change:**

When the agent wants to modify an existing file:
1. Identify which `hick:file` element in the pipeline owns that file path
2. Identify the relevant `hick:paste` slot(s) in that file (by name, via PasteSlotRegistry from TASK-17)
3. Produce a `hick:copy target="<paste-slot-name>"` element with the new content
4. Append the `hick:copy` to the pipeline (via `_hick.yml` or the owning `.hick` file)
5. Run `hick run` to verify the output is correct

When the agent wants to create a new file:
1. Produce a `hick:file path="..."` element with the file content
2. Append to pipeline
3. Run `hick run` to verify

**When to fall back to direct write:**
- Files with no `hick:file` owner in the pipeline (external/untracked files)
- Binary files
- The agent should log a warning when falling back

**Key dependency on TASK-17:** Paste slots must exist in target files for this to work.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Agent modifies existing pipeline-owned files via hick:copy elements, not direct writes
- [ ] #2 Agent creates new files via hick:file elements appended to the pipeline
- [ ] #3 Agent falls back to direct write for files not owned by the pipeline, with a logged warning
- [ ] #4 hick run is called after each pipeline edit to verify correctness
- [ ] #5 Session file records the hick:copy/hick:file elements produced (not the raw file content)
- [ ] #6 Two agents running simultaneously on different branches each produce hick:copy edits that merge without conflict (manual test)
- [ ] #7 cargo test --workspace passes
<!-- AC:END -->
