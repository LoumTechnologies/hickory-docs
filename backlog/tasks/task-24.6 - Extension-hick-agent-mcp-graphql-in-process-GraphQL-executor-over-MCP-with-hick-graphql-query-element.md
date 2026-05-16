---
id: TASK-24.6
title: >-
  Extension hick-agent-mcp-graphql: in-process GraphQL executor over MCP with
  hick:graphql-query element
status: To Do
assignee: []
created_date: '2026-05-16 12:23'
updated_date: '2026-05-16 12:57'
labels:
  - mcp
  - extensions
  - graphql
dependencies:
  - TASK-24.1
  - TASK-24.2
  - TASK-24.7
references:
  - hick-agent/crates/hick-agent-hick-edit/src/lib.rs
  - CLAUDE.md (XML entity escaping section)
parent_task_id: TASK-24
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
New extension crate `hick-agent-mcp-graphql`. The LLM writes a `<hick:graphql-query>` element containing a GraphQL query or mutation. The host extension parses the element, executes the query against an in-process GraphQL schema backed by `McpClient`, and injects the result into the conversation as a user-role observation. No network, no interceptor, no magic IP — structurally identical to TASK-24.5's `<hick:tool>` pattern.

## Element schema

```xml
<hick:graphql-query>
mutation {
  githubCreateIssue(owner: "myorg", repo: "myrepo", title: "Fix bug", body: "...") {
    number
    url
  }
}
</hick:graphql-query>
```

CDATA wrapping is required when the query contains `<` or `&` (see CLAUDE.md XML entity escaping note).

## How it works

1. **Response scanning** — After each LLM response is assembled, the extension scans for `<hick:graphql-query>` elements, same pattern as `<hick:tool>` in TASK-24.5.
2. **In-process execution** — The GraphQL query is executed against a schema built from `Vec<McpToolDef>` using an in-process executor (e.g. `async-graphql`). Each resolver calls `ToolGate::check` then `McpClient::call`. Field selection is enforced by the executor — only selected fields appear in the result.
3. **Result injection** — Result is appended to the conversation as a user-role message. Format is determined by TASK-24.7's evaluation outcome (JSON, Markdown, or XML).
4. **Auth isolation** — Credentials stay in `McpCredentialStore` inside the host process. The LLM only sees the result.

## Why GraphQL over plain `<hick:tool>`

`<hick:tool>` always returns the full tool response. `<hick:graphql-query>` lets the LLM select exactly which fields enter the context — important for tools with large responses (file trees, PR objects, search results). The LLM can also call multiple tools in a single element using GraphQL aliases.

## Schema generation

Each `McpToolDef` → one mutation or query field, arguments from input JSON Schema, output type fields from output JSON Schema (fallback: `result: String`). Introspection enabled so the LLM can discover available tools.

## Scope boundaries
- No HTTP server, no port binding, no interceptor.
- Schema hot-reloads when `McpClient::refresh()` is called.
- Subscriptions are out of scope for v1.
- Response format (JSON / Markdown / XML) is decided by TASK-24.7 and implemented here.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 LLM response containing a <hick:graphql-query> element causes the extension to execute the query in-process and inject a result observation into the conversation.
- [ ] #2 No network socket of any kind is opened — the GraphQL executor runs entirely in the host Rust process.
- [ ] #3 Field selection is enforced: querying { number url } from a tool that also returns title and body does not include title or body in the result.
- [ ] #4 Multiple tools queried in a single element via GraphQL aliases each call ToolGate::check and McpClient::call independently.
- [ ] #5 ToolGate::check is called inside each resolver; a blocked resolver returns a GraphQL error for that field without aborting the whole query.
- [ ] #6 Auth credentials do not appear in the result observation or anywhere in conversation history.
- [ ] #7 A classification=secret context causes resolvers to block their calls (integration test).
- [ ] #8 GraphQL introspection is available: the LLM can send an introspection query as a <hick:graphql-query> and receive the full schema.
- [ ] #9 Response format matches the outcome of TASK-24.7.
- [ ] #10 The hick-edit extension ignores <hick:graphql-query> elements — no pipeline corruption.
- [ ] #11 Extension integrates into an agent binary without source changes to hick-agent-sdk or hick-agent-hick-edit.
<!-- AC:END -->
