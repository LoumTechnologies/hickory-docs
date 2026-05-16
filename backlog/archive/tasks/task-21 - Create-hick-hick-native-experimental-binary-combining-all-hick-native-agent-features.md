---
id: TASK-21
title: >-
  Create hick-hick-native: experimental binary combining all hick-native agent
  features
status: To Do
assignee: []
created_date: '2026-05-15 12:47'
labels: []
milestone: m-1
dependencies:
  - TASK-16
  - TASK-17
  - TASK-18
  - TASK-19
  - TASK-20
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Create the `hick-hick-native` binary crate that wires together `hick-agent-sdk` + `hick-agent-script-first` + `hick-agent-hick-edit` into the full-vision hick-native agent. This is the experimental agent to evaluate against `hick-basic`.

**What hick-hick-native does differently from hick-basic:**
- Edits via hick:copy / hick:file (TASK-18) instead of direct file writes
- Uses paste slots as composition points (TASK-17)
- Reads the pipeline on startup to understand repo state (TASK-20)
- Can run `hick promote` at session end to clean up the pipeline (TASK-19)
- SYSTEM_PROMPT is extended with hick pipeline awareness: "You are working in a hickory docs repo. To modify files, produce hick:copy elements..."

**Evaluation questions this binary helps answer:**
1. Does the agent produce better output when it uses hick:copy vs direct writes?
2. Does paste-slot awareness lead to cleaner, more composable code?
3. Do concurrent agent sessions on branches merge more cleanly?
4. Is session → pipeline promotion valuable in practice?

**Crate location:** `hick-agent/crates/hick-hick-native/` with `[[bin]] name = "hick-hick-native"`

**Comparison harness (stretch goal):**
A script that runs `hick-basic` and `hick-hick-native` on the same task, then compares outputs. Implement only if both binaries are stable enough to run head-to-head.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 hick-hick-native binary exists and compiles
- [ ] #2 Binary uses hick-agent-sdk + hick-agent-script-first + hick-agent-hick-edit
- [ ] #3 Agent reads pipeline on startup and uses PasteSlotRegistry for context
- [ ] #4 Agent produces hick:copy edits and hick:file elements during a coding session
- [ ] #5 SYSTEM_PROMPT includes hick pipeline editing instructions
- [ ] #6 End-to-end test: hick-hick-native completes a coding task and the resulting pipeline reproduces the output via hick run
- [ ] #7 cargo test --workspace passes
<!-- AC:END -->
