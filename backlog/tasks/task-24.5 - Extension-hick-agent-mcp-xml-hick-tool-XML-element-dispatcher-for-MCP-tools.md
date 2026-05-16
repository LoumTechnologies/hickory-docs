---
id: TASK-24.5
title: 'Extension hick-agent-mcp-xml: hick:tool XML element dispatcher for MCP tools'
status: To Do
assignee: []
created_date: '2026-05-16 12:23'
updated_date: '2026-05-16 13:02'
labels:
  - mcp
  - extensions
  - hick-xml
milestone: m-3
dependencies:
  - TASK-24.1
  - TASK-24.2
references:
  - hick-agent/crates/hick-agent-hick-edit/src/lib.rs
  - CLAUDE.md (XML entity escaping section)
parent_task_id: TASK-24
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
New extension crate `hick-agent-mcp-xml`. Introduces a `<hick:tool>` XML element that the LLM writes directly in its response. The extension parses these elements, dispatches them to the appropriate MCP server, and returns results as structured observations. This mode requires no script execution — the tool call is declarative XML.

## Element schema

```xml
<hick:tool server="github" name="create_issue">
  <hick:arg name="owner">myorg</hick:arg>
  <hick:arg name="repo">myrepo</hick:arg>
  <hick:arg name="title">Fix the bug</hick:arg>
  <hick:arg name="body">Describe the bug...</hick:arg>
</hick:tool>
```

Or with a JSON body for tools with complex inputs:

```xml
<hick:tool server="filesystem" name="write_file">
  <hick:input><![CDATA[{"path": "/workspace/out.txt", "content": "hello"}]]></hick:input>
</hick:tool>
```

The `hick:input` CDATA form is required for inputs containing `<`, `>`, or `&` (see CLAUDE.md XML entity escaping note).

## How it works

1. **Response scanning** — After each LLM response chunk is assembled, the extension scans for `<hick:tool>` elements. Found elements are extracted from the text before it is appended to conversation history (the raw LLM response is stored; tool results are injected as a subsequent user-role observation message).

2. **Dispatch** — For each element, parse `server` and `name` attributes, build the JSON input object from child elements or `<hick:input>` CDATA. Call `ToolGate::check`, then `McpClient::call("tools/call", ...)`.

3. **Result injection** — Tool results are appended to the conversation as a user-role message in a structured XML format:
   ```xml
   <hick:tool-result server="github" name="create_issue" status="ok">
     <![CDATA[{"number": 42, "url": "https://github.com/..."}]]>
   </hick:tool-result>
   ```
   The LLM then continues its response in the next turn with the result in context.

4. **Auth isolation** — No credentials appear in the XML elements or in conversation history. The extension calls `McpClient` (which holds credentials via `McpCredentialStore`) internally.

5. **System prompt** — Extension emits a system prompt section documenting the `<hick:tool>` element schema and listing available tools with their argument names.

## Compatibility with hick-agent-hick-edit
`<hick:tool>` elements must not be mistaken for pipeline-affecting elements by the hick-edit extension. The XML namespace prefix (`hick:tool` vs `hick:copy`, `hick:paste`) distinguishes them at the parser level. Confirm the hick-lang XML parser does not reject or misinterpret unknown `hick:` elements.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 LLM response containing a <hick:tool> element causes the extension to dispatch a tools/call to the MCP server and inject a <hick:tool-result> observation.
- [ ] #2 hick:arg children and hick:input CDATA are both parsed correctly and produce the expected JSON input object.
- [ ] #3 ToolGate::check is called before dispatch; blocked calls inject a <hick:tool-result status='blocked'> observation instead of calling the server.
- [ ] #4 Auth credentials do not appear in any conversation history message.
- [ ] #5 A classification=secret context causes the call to be blocked (integration test).
- [ ] #6 The hick-edit extension (hick-agent-hick-edit) ignores <hick:tool> and <hick:tool-result> elements — no pipeline corruption.
- [ ] #7 System prompt fragment documents the element schema and all available tool names/arguments.
- [ ] #8 Extension integrates into an agent binary without source changes to hick-agent-sdk or hick-agent-hick-edit.
<!-- AC:END -->
