# copper-ledge

A/B evaluation binary — the "standard" agent baseline with no Hickory pipeline
knowledge.

Wires `hick-agent-sdk` + `hick-agent-script-first` + `hick-agent-core` and
launches a ratatui TUI with the script-first ReAct loop. Also includes MCP
server management subcommands (`mcp add`, `mcp remove`, `mcp list`).

The dependency manifest **deliberately excludes** `hick-agent-hick-edit` so
that comparisons against `iron-peak` measure the value of pipeline-native
editing in isolation.

See `iron-peak` for the counterpart binary that adds pipeline integration.
