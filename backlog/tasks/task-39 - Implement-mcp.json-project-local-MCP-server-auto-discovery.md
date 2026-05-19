---
id: TASK-39
title: Implement mcp.json project-local MCP server auto-discovery
status: To Do
assignee: []
created_date: '2026-05-19 17:28'
labels:
  - agent-standards
  - mcp
dependencies: []
references:
  - 'https://spec.modelcontextprotocol.io'
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Support the `mcp.json` / `.mcp.json` standard for project-local MCP server declarations. When the agent starts in a project directory, it should automatically discover and connect to any MCP servers declared in the project's config file.

This is the de-facto cross-agent convention (used by Claude Code, Cursor, and others) for a project to ship its own MCP tooling without requiring manual configuration by each developer.

**File locations to check (in priority order):**
1. `.mcp.json` at project root
2. `mcp.json` at project root

**Schema (OpenAI/Anthropic convention):**
```json
{
  "mcpServers": {
    "server-name": {
      "command": "npx",
      "args": ["-y", "@some/mcp-server"],
      "env": { "API_KEY": "${MY_API_KEY}" }
    }
  }
}
```

**Scope:**
- Detect and parse `mcp.json` / `.mcp.json` at startup
- Merge declared servers into the active MCP client config
- Support `command`/`args`/`env` fields; env values support `${VAR}` expansion from the process environment
- Servers from the project file augment (not replace) any user-level MCP config

**References:**
- `hick-agent/crates/hick-agent-sdk/src/mcp_config.rs` — MCP configuration
- `hick-agent/crates/hick-agent-sdk/src/mcp_client.rs` — MCP client
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 mcp.json and .mcp.json at project root are detected on startup
- [ ] #2 Declared servers are connected alongside user-level MCP config
- [ ] #3 Environment variable interpolation (${VAR}) works in env fields
- [ ] #4 Missing or malformed file logs a warning and does not crash startup
- [ ] #5 Tests cover: server discovery, env interpolation, missing file no-op, malformed JSON warning
<!-- AC:END -->
