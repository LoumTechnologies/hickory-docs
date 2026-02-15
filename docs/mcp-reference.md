# MCP Reference

hick-mcp is an MCP (Model Context Protocol) server that exposes hick lens and
exec operations as tools for AI coding agents. It runs over stdio and integrates
with Claude Code or other MCP-aware clients.

hick-mcp depends on hick-lens for tree-sitter AST analysis, selectors, and
source edit computation. It depends on hick (the main pipeline crate) for
executing `.hick` files.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Running the Server](#2-running-the-server)
3. [ServerState](#3-serverstate)
4. [Tools: Pipeline](#4-tools-pipeline)
5. [Tools: Lens (Read)](#5-tools-lens-read)
6. [Tools: Lens (Write)](#6-tools-lens-write)
7. [Tools: Container](#7-tools-container)
8. [Typical Workflow](#8-typical-workflow)

---

## 1. Overview

The server implements the MCP protocol (version `2025-03-26`) using the `rmcp`
crate. It exposes seven tools grouped into four categories:

| Category | Tools | Purpose |
|----------|-------|---------|
| Pipeline | `hick_run`, `hick_preview` | Run the hick pipeline and preview output files |
| Lens (read) | `hick_query`, `hick_structure`, `hick_provenance` | Explore output structure and provenance |
| Lens (write) | `hick_replace` | Compute source edits to modify `.hick` files |
| Container | `hick_exec` | Run shell commands in sandboxed WASM Linux containers |

All state is held in `ServerState`, an async-safe cache that stores source
files, pipeline outputs, provenance maps, tree-sitter AST mappings, and
container executor state.

---

## 2. Running the Server

**Crate:** `hick-mcp` (binary)

The server communicates over stdio. Stdout is reserved for the MCP protocol;
logging goes to stderr.

```bash
# Lens and pipeline tools only
hick-mcp

# With container execution enabled
hick-mcp --images-dir /path/to/wasm-images
```

### CLI Arguments

| Flag | Required | Default | Description |
|------|----------|---------|-------------|
| `--images-dir` | no | -- | Directory containing `.wasm` container images. Enables `hick_exec` |
| `--workdir` | no | CWD | Working directory to mount in containers at `/workspace` |
| `--default-image` | no | `alpine` | Default container image name |

### Claude Code Integration

Add to your MCP configuration (`.claude/mcp.json` or similar):

```json
{
  "mcpServers": {
    "hick": {
      "command": "hick-mcp",
      "args": ["--images-dir", "/path/to/wasm-images"]
    }
  }
}
```

See [Claude Code Setup](claude-code-setup.md) for full configuration
instructions including sandboxed-only execution.

### Server Info

| Field | Value |
|-------|-------|
| Protocol version | `2025-03-26` |
| Capabilities | Tools |
| Instructions | "Use hick_run first to load sources, then use hick_query/hick_structure/hick_provenance to explore output." |

---

## 3. ServerState

**Module:** `hick_mcp::state`

Thread-safe cached state for the MCP server. Backed by
`Arc<Mutex<Inner>>` (tokio mutex).

### Cached Data

| Field | Type | Description |
|-------|------|-------------|
| `sources` | `HashMap<String, String>` | Source `.hick` files (name -> content) |
| `files` | `HashMap<String, FileContent>` | Cached file outputs from last pipeline run |
| `provenance_maps` | `HashMap<String, ProvenanceMap>` | Cached provenance maps from last pipeline run |
| `ast_mappings` | `HashMap<String, AstMapping>` | Cached tree-sitter AST mappings per output file |
| `run_summary` | `Option<RunSummary>` | Summary of last pipeline run |

### Key Methods

| Method | Description |
|--------|-------------|
| `load_sources` | Load `.hick` source files. Clears all cached results |
| `run_pipeline` | Run the pipeline, build AST mappings for recognised output files, cache everything |
| `get_ast_mapping` | Get AST mapping for a specific output file |
| `get_provenance_map` | Get provenance map for a specific output file |
| `get_file_content` | Get text content of an output file |

### Caching Behaviour

- `load_sources` invalidates all cached results (files, provenance maps, AST
  mappings, run summary).
- `run_pipeline` automatically builds `AstMapping` for every text output file
  whose extension is recognised by `Language::from_path`.

---

## 4. Tools: Pipeline

### hick_run

Run the hick pipeline on source files and cache results.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `sources` | `Vec<SourceFile>` | yes | Source `.hick` files as name/content pairs |
| `params` | `Vec<String>` | no | Pipeline parameters as `key=value` strings |

**SourceFile:**

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | File name or path |
| `content` | `String` | File content |

**Response:** A summary listing the number of files generated and per-file
details (path, size, provenance span count).

**Example call:**

```json
{
  "tool": "hick_run",
  "arguments": {
    "sources": [
      { "name": "app.hick", "content": "<hick:file path=\"src/main.rs\">fn main() {}</hick:file>" }
    ],
    "params": ["env=prod"]
  }
}
```

**Example response:**

```
1 file(s) generated:
  src/main.rs (14 bytes, 1 provenance spans)
```

---

### hick_preview

Preview the rendered content of a specific output file.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `file` | `String` | yes | Output file path to preview |

**Response:** The full text content of the output file.

---

## 5. Tools: Lens (Read)

### hick_query

Query output files for structural elements matching a selector.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `selector` | `String` | yes | Selector string (e.g. `fn main`, `struct Config`, `#id`, `.class`) |
| `file` | `String` | no | Specific output file to search in. Searches all files if omitted |

**Response:** For each match, shows the element kind, name, file, line range,
output text, and provenance origins.

**Example call:**

```json
{
  "tool": "hick_query",
  "arguments": {
    "selector": "fn main",
    "file": "src/main.rs"
  }
}
```

**Example response:**

```
=== Function 'main' in src/main.rs [L10-L15] ===
fn main() {
    let config = Config::default();
    run(config);
}
--- Origins: Literal { file: "app.hick", span: SourceSpan { start: 100, end: 200 } }
```

---

### hick_structure

Get the structural summary of an output file.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `file` | `String` | yes | Output file path |
| `depth` | `usize` | no | Maximum nesting depth |

**Response:** A tree-structured text summary showing structural elements with
line ranges.

**Example response:**

```
fn main [L1-L5]
fn helper [L7-L10]
struct Config [L12-L15]
```

Requires `hick_run` to have been called first to populate the AST mapping
cache.

---

### hick_provenance

Get provenance information for a line range in an output file.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `file` | `String` | yes | Output file path |
| `start_line` | `usize` | yes | Start line (1-based) |
| `end_line` | `usize` | yes | End line (1-based, inclusive) |

**Response:** For each provenance span overlapping the given line range, shows
the byte range, origin, and a text preview (truncated to 80 characters).

**Example response:**

```
[0-42] Literal { file: "app.hick", span: SourceSpan { start: 10, end: 52 } }
  Text: "fn main() {\n    let config = Config::default();\n"
```

---

## 6. Tools: Lens (Write)

### hick_replace

Compute source edits to replace an element in the `.hick` source. Returns the
edits that would be applied but does not modify files.

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `selector` | `String` | yes | Selector for element to replace (e.g. `fn greet`) |
| `file` | `String` | yes | Output file to search in |
| `new_content` | `String` | yes | New content to replace the matched element with |

**Response:** For each computed edit, shows the confidence level (`EXACT`,
`APPROXIMATE`, or `UNMAPPED`), the `.hick` source file path, the byte range,
and the replacement content.

**Example call:**

```json
{
  "tool": "hick_replace",
  "arguments": {
    "selector": "fn greet",
    "file": "src/main.rs",
    "new_content": "fn greet() {\n    println!(\"hello world\");\n}"
  }
}
```

**Example response:**

```
Edit [EXACT]: app.hick bytes [100-114]
  Replace with: fn greet() {
    println!("hello world");
}
```

### Confidence Levels

| Level | Meaning |
|-------|---------|
| `EXACT` | Provenance maps directly to a `.hick` literal span. Safe to apply automatically. |
| `APPROXIMATE` | The element spans mixed provenance origins. The edit targets the enclosing hick region and may need review. |
| `UNMAPPED` | No provenance data. The edit cannot be mapped back to source. |

---

## 7. Tools: Container

### hick_exec

Run a shell command in a sandboxed WASM Linux container. Requires the server to
be started with `--images-dir`.

The host working directory is mounted at `/workspace` inside the container.
Containers persist across calls within a session, preserving installed packages,
environment variables, and filesystem state.

**Parameters:**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `command` | `String` | yes | -- | Shell command to run |
| `image` | `String` | no | server default | Container image name |
| `container` | `String` | no | `"workspace"` | Container name for session persistence |
| `timeout` | `u64` | no | 120 | Timeout in seconds |

**Response:** The captured stdout/stderr output from the command.

**Example call:**

```json
{
  "tool": "hick_exec",
  "arguments": {
    "command": "cargo test --workspace",
    "timeout": 300
  }
}
```

**Example response:**

```
   Compiling my-crate v0.1.0
    Finished test [unoptimized + debuginfo] target(s)
     Running unittests src/lib.rs
test result: ok. 12 passed; 0 failed; 0 ignored
```

**Session persistence:**

Containers remain running between calls. To use separate environments:

```json
{"tool": "hick_exec", "arguments": {"command": "apk add nodejs npm", "container": "frontend"}}
{"tool": "hick_exec", "arguments": {"command": "npm test", "container": "frontend"}}
{"tool": "hick_exec", "arguments": {"command": "cargo build", "container": "backend"}}
```

---

## 8. Typical Workflow

A typical interaction with hick-mcp follows this sequence:

1. **Load and run** -- Call `hick_run` with `.hick` source files and any
   pipeline parameters. This executes the pipeline, caches all output files
   with their provenance maps, and builds tree-sitter AST mappings.

2. **Explore structure** -- Call `hick_structure` on an output file to see its
   top-level elements (functions, structs, classes, etc.) with line ranges.

3. **Query elements** -- Call `hick_query` with a selector to find specific
   elements across one or all output files. The response includes the element
   source text and its provenance origins.

4. **Inspect provenance** -- Call `hick_provenance` on a line range to see
   exactly which `.hick` source positions produced that region of output.

5. **Preview output** -- Call `hick_preview` to see the full rendered content
   of any output file.

6. **Compute edits** -- Call `hick_replace` with a selector and new content
   to compute the `.hick` source byte ranges that need to change. Review the
   confidence level before applying.
