# hick-mcp

Standalone MCP server binary (`hick-mcp` executable) that exposes hick lens and
exec operations as MCP tools for use by Claude Code or other MCP-aware clients.

Uses the `rmcp` crate over stdio transport. Sandboxed container execution is
enabled when `--images-dir` is supplied.

## Exposed tools

- Query generated-output structure
- Map generated file lines back to `.hick` source positions
- Compute source edits for a given selector
- Run the hick pipeline

Gives external AI clients the same structural query/edit capabilities that
`hick-agent-hick-edit` gives the internal agents.
