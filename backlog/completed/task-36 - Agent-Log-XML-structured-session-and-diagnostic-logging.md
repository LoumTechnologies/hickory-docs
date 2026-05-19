---
id: TASK-36
title: 'Agent Log XML: structured session and diagnostic logging'
status: Done
assignee:
  - nate@loumtechnologies.com
created_date: '2026-05-19 11:00'
updated_date: '2026-05-19 11:32'
labels:
  - agent
  - logging
  - architecture
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace the current ad-hoc `hick:`-namespaced session files and `env_logger` plain-text diagnostic output across all agent crates with a unified Agent Log XML format.

## Background

Agent session logs in `iron-peak` and `copper-ledge` currently write `<hick:session>` elements, incorrectly borrowing the Hickory pipeline namespace. Diagnostic logging uses `env_logger` (plain text). Neither is consistent, and both are written by the binary crates — meaning every binary must independently get logging right.

## Goals

- Define a purpose-built **Agent Log XML** format: `xmlns:log="http://www.hickorydocs.com/agent-log/1.0"`
- A `SessionLog` trait in `hick-agent-sdk` makes the SDK and extension crates responsible for logging correctness — binary crates only supply a writer path at startup
- Session logs become directory bundles (main `session.xml` + attachment files) so extensions can log binary artifacts (images, generated files) without encoding them in XML
- MCP tool calls and responses become first-class session log events (currently not recorded anywhere)
- `env_logger` plain-text diagnostic output replaced with XML-formatted log lines across all agent binaries

## Out of scope

- `hick-mcp` (MCP server binary) — not an agent, has no session concept
- State machine changes — states remain closed: `Idle`, `Thinking`, `Executing`, `Observing`, `Done`, `Error`
- The `decide` response protocol intent (separate feature)
<!-- SECTION:DESCRIPTION:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
## Implementation Plan

### 36.1 — session_log.rs in hick-agent-sdk
- LogLevel, LogAttachment, LogEvent<'a> (StateTransition/ActionEnd use &str/plain fields to avoid circular deps), SessionLog trait, NullSessionLog
- Re-export from lib.rs

### 36.2 — xml_session_log.rs in hick-agent-sdk
- XmlSessionLog::new(dir) creates bundle dir + session.xml, writes XML declaration + root element
- record() dispatches variants to log: elements with time attr, flushes after each write
- Standard XML escaping for text content
- XmlDiagnosticLogger: implements log::Log, forwards to Arc<dyn SessionLog>
- Unit tests

### 36.3 — Wire into ReactLoops + LoggingMcpClient
- hick-agent-script-first and hick-agent-engine ReactLoops get session_log field + with_session_log() builder
- Emit SessionStart, UserTurn, StateTransition, LlmResponse, ActionStart, ActionEnd, SecurityViolation, SessionEnd at correct points
- LoggingMcpClient<C> wrapper added to hick-agent-sdk

### 36.4 — Remove TUI session writing; replace env_logger
- Remove log_* methods, log fields, xml_escape from iron-peak and copper-ledge TUI structs
- Remove log_path from run_tui() signatures
- Wire XmlSessionLog + XmlDiagnosticLogger in all 4 binary main.rs files
- Remove env_logger from all 4 Cargo.toml files
- cargo test --workspace -- --test-threads=1
<!-- SECTION:PLAN:END -->
