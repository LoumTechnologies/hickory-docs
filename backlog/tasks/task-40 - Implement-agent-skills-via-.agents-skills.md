---
id: TASK-40
title: Implement agent skills via .agents/skills/
status: To Do
assignee: []
created_date: '2026-05-19 17:28'
labels:
  - agent-standards
  - skills
dependencies: []
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Support a skills system where projects (and users) can define reusable, invocable prompts stored as Markdown files under `.agents/skills/`. Skills are named prompt templates the agent can invoke by name — similar to Claude Code's slash commands or Cursor's prompt files.

**Convention:**
- Skills live in `.agents/skills/*.md` at the project root
- Each file is a skill; the filename (without `.md`) is the skill name
- The file body is the prompt template, optionally with `$ARGUMENTS` placeholder for inline arguments
- The agent exposes skills as slash-commands or tool calls: `/skill-name [args]`

**Scope:**
- Load skill files from `.agents/skills/` in `ProjectContextLoader` (or a new `SkillLoader`)
- Expose loaded skills to the runtime so they can be invoked by name
- Support `$ARGUMENTS` substitution when a skill is invoked with arguments
- List available skills when the user types `/skills` or asks what skills are available

**Example skill file (`.agents/skills/summarise.md`):**
```
Summarise the following in three bullet points:

$ARGUMENTS
```

**References:**
- `hick-agent/crates/hick-agent-sdk/src/project_context.rs` — context loader
- `hick-agent/crates/hick-agent-sdk/src/runtime.rs` — runtime loop
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Skills in .agents/skills/*.md are discovered at startup
- [ ] #2 Skills can be invoked by name from the agent conversation
- [ ] #3 $ARGUMENTS is substituted with inline arguments when provided
- [ ] #4 Listing skills shows names and first line of each skill file as a description
- [ ] #5 Tests cover: skill discovery, invocation with args, invocation without args, empty skills dir
<!-- AC:END -->
