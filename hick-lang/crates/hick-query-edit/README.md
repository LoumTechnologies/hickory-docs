# hick-query-edit

CLI utility (`hick-query-edit` executable) for inspecting and editing hick
pipeline outputs from the command line.

## Subcommands

| Subcommand | Description |
|---|---|
| `parse` | Dump a `.hick` file as JSON AST |
| `query` | Structural selector search over generated output + provenance |
| `structure` | Summary of structural elements in generated output |
| `provenance` | Map a line range in generated output back to its `.hick` source |
| `compute-edits` | Produce a JSON edits file for a given selector replacement |
| `apply-edits` | Apply computed edits to `.hick` source files |
| `pipeline-list-owned` | List all pipeline-managed output files |

A command-line companion to the `HickEditService` logic in
`hick-agent-hick-edit`.
