# hick-agent-mcp-graphql

MCP extension that lets the LLM call MCP tools by writing
`<hick:graphql-query>` elements in its response.

`McpGraphqlExtension` scans for those elements, executes each against an
in-process `async-graphql` dynamic schema (no network port, no interceptor), and
injects results as `<hick:graphql-result>` user-role observations.

Each `McpToolDef` becomes a mutation field. Every resolver calls
`ToolGate::check` before dispatching to `McpClient::call`. GraphQL introspection
is enabled so the LLM can discover available tools at runtime by querying the
schema.
