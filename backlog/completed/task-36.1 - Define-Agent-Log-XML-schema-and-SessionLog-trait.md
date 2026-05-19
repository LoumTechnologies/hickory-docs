---
id: TASK-36.1
title: Define Agent Log XML schema and SessionLog trait
status: Done
assignee:
  - nate@loumtechnologies.com
created_date: '2026-05-19 11:00'
updated_date: '2026-05-19 11:12'
labels:
  - agent
  - logging
dependencies: []
parent_task_id: TASK-36
priority: high
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Define the Agent Log XML format and the `SessionLog` trait in `hick-agent-sdk`. This is the foundational task — all other logging tasks depend on the types defined here.

## Format

- Namespace: `xmlns:log="http://www.hickorydocs.com/agent-log/1.0"`
- Session log is a **directory bundle**: a `session.xml` root document plus optional attachment files referenced by relative path
- State machine config and runtime audit share the same schema

## SessionLog trait

Single-method trait so any backend (file writer, no-op, test spy) can implement it:

```rust
pub trait SessionLog: Send + Sync {
    fn record(&self, event: LogEvent);
}
```

`LogEvent` is an enum with typed core variants plus a `Custom` escape hatch for extensions:

```rust
pub enum LogEvent<'a> {
    SessionStart { ... },
    UserTurn { text: &'a str },
    StateTransition { from: &'a AgentState, to: &'a AgentState },
    LlmResponse { text: &'a str },
    ActionStart { lang: &'a str, code: &'a str },
    ActionEnd { result: &'a ScriptResult },
    McpCall { server: &'a str, method: &'a str, params: &'a serde_json::Value },
    McpResponse { server: &'a str, method: &'a str, result: &'a serde_json::Value },
    SecurityViolation { description: &'a str },
    DiagnosticLog { level: LogLevel, target: &'a str, message: &'a str },
    SessionEnd,
    Custom { xml_fragment: &'a str, attachments: &'a [LogAttachment] },
}

pub struct LogAttachment {
    pub filename: String,   // relative path within session bundle dir
    pub data: Arc<[u8]>,
}
```

## XML element design

Sketch the element shapes for all core variants. Every element carries a `time` attribute (RFC 3339). Example:

```xml
<log:session xmlns:log="http://www.hickorydocs.com/agent-log/1.0" start="...">
  <log:state-transition from="Idle" to="Thinking" time="..."/>
  <log:user time="...">user message text</log:user>
  <log:assistant time="...">llm response text</log:assistant>
  <log:action lang="python" time="...">code here</log:action>
  <log:observation exit="0" time="...">stdout here</log:observation>
  <log:mcp-call server="github" method="tools/call" time="...">...params as child element...</log:mcp-call>
  <log:mcp-response server="github" method="tools/call" time="...">...result...</log:mcp-response>
  <log:diagnostic level="info" target="hick_agent_sdk::react_loop" time="...">message</log:diagnostic>
  <log:session-end time="..."/>
</log:session>
```

## Crate location

New module `hick-agent-sdk/src/session_log.rs`. Export `SessionLog`, `LogEvent`, `LogAttachment`, `LogLevel` from `hick-agent-sdk::lib`.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 SessionLog trait, LogEvent enum, LogAttachment, and LogLevel are defined in hick-agent-sdk::session_log and re-exported from lib.rs
- [x] #2 LogEvent has typed variants for all core agent events: session start/end, user turn, state transition, LLM response, action start/end, MCP call/response, security violation, diagnostic log
- [x] #3 LogEvent::Custom carries an xml_fragment string and a slice of LogAttachment, allowing extensions to log arbitrary XML elements with binary file attachments
- [x] #4 All types are Send + Sync
- [x] #5 A no-op SessionLog impl (NullSessionLog) is provided for use in tests and contexts where logging is not needed
- [x] #6 Module has doc comments explaining the format, namespace URI, and bundle directory layout
<!-- AC:END -->
