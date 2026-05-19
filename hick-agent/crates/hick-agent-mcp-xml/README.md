# hick-agent-mcp-xml

MCP extension for purely declarative tool calls via XML elements.

The LLM writes `<hick:tool server="…" name="…">` elements with either
`<hick:arg>` children or a `<hick:input>` CDATA block.
`McpXmlExtension::process_response` scans for those elements, calls
`ToolGate::check`, dispatches to `McpClient::call`, and returns results as
`<hick:tool-result>` XML observations.

No script execution is involved. Coexists safely with `hick-agent-hick-edit`
because the element names do not conflict with any pipeline-affecting hick
elements.
