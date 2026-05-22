# Directory and Crate Organization

The repo uses subsystem directories (`hick-agent/`, `hick-exec/`, `wasm-net/`,
etc.) to group related crates. The top-level `crates/` directory is an
informal catch-all that has accumulated unrelated things. A few crates are
housed in the wrong subsystem.

---

## 1. `crates/` is a heterogeneous catch-all

**Current contents:**

| Crate | What it actually is |
|-------|---------------------|
| `hickory-xml` | XML DOM abstraction (pipeline primitive) |
| `hick-case` | Text case transforms (pipeline primitive) |
| `hick-feature` | Feature flag system (pipeline primitive) |
| `hick-condition` | Boolean expression eval (pipeline primitive) |
| `hick-handlers` | Pipeline step handlers |
| `hick-literate` | Literate hick pipeline (the core pipeline runner) |
| `hick-live` | Streaming Arrow/Parquet pipeline runtime |
| `hick-lens` | Tree-sitter structural query for hick files |
| `hick-api` | HTTP API interceptor served inside WASM containers |
| `hick-live` | Streaming reactive pipeline (Arrow/Parquet) |
| `bench` | Benchmarks |
| `wasm-test-tool` | WASM dev tool |
| `wasi-example` | WASI example |

### Option A — Split `crates/` by destination (recommended)

Move misplaced crates to the subsystem that owns them:

| Crate | Move to | Rationale |
|-------|---------|-----------|
| `hick-live` | `hick-reactive/crates/` | It IS a reactive pipeline runtime; same subsystem as `hick-flow` and `hick-grove` |
| `hick-api` | `hick-container/crates/` | It intercepts HTTP inside the container network stack, not the pipeline |
| `hick-lens` | `hick-lang/crates/` | Tree-sitter structural queries are language tooling |
| `bench` | `crates/bench/` → rename `hick-bench`, keep in `crates/` | Benchmarks can stay but should be named |
| `wasm-test-tool` | `hick-container/crates/` or top-level `examples/` | Not a production library |
| `wasi-example` | Top-level `examples/` | Example code, not a library |

Remaining `crates/` after the moves:
`hickory-xml`, `hick-case`, `hick-feature`, `hick-condition`, `hick-handlers`,
`hick-literate`, `hick-bench` — all pipeline-layer primitives. This is a
coherent group.

### Option B — Leave `crates/` as-is

Accept it as a heterogeneous primitive bucket and only enforce naming (doc #1).
Lower effort; the logical inconsistency remains but is less painful once names
are clear.

---

## 2. `hick-query-edit` belongs in `hick-lang/` or `hick-exec/`, not `hick-agent/`

**Location:** `hick-agent/crates/hick-query-edit/`

`hick-query-edit` is a standalone CLI tool that queries and edits hick files.
Its dependencies are `hick-lang`, `hick-lens`, and `hick-flow` — no
`hick-agent-sdk` or agent-specific code. It landed in `hick-agent/crates/`
because it was built as part of agent tooling evaluation, but conceptually it
is a pipeline developer tool.

### Option A — Move to `hick-lang/crates/` (recommended)

`hick-query-edit` joins `hick-lang` and `hick-lsp` as language tooling. The
binary is useful to any hick developer, not just agent users.

### Option B — Move to `hick-exec/crates/`

It fits there too since `hick-exec` covers pipeline execution. Either location
is better than `hick-agent/`.

---

## 3. `hick-mcp` is pipeline infrastructure, not agent-internal

**Location:** `hick-agent/crates/hick-mcp/`

`hick-mcp` is a standalone MCP server binary that exposes the hick pipeline
over the Model Context Protocol. It depends on `hick-literate`, `hick-lens`,
`hick-flow`, `hick-lang`, and `hick-exec` — no `hick-agent-*` crates.

It lives in `hick-agent/crates/` for historical reasons (it was built to serve
the agent) but it is infrastructure that any MCP client could use.

### Option A — Move to `hick-lang/crates/` (recommended)

`hick-mcp` alongside `hick-lsp` makes the "language tooling and servers"
subsystem coherent: LSP for IDE editors, MCP for LLM clients.

### Option B — Create `hick-servers/crates/`

A new `hick-servers/` subsystem collecting `hick-mcp`, `hick-lsp`,
`hick-relay`, and `hick-grove-server`. Higher overhead (new directory) but
makes the "server binaries" category explicit.

---

## 4. `hick-relay` in `hick-agent/crates/`

**Location:** `hick-agent/crates/hick-relay/`

`hick-relay` is a cloud WebSocket relay server for forwarding UI events between
agents and browsers. It's server infrastructure, not agent-local code.

If Option B from item 3 is chosen (create `hick-servers/`), move `hick-relay`
there. Otherwise it can stay in `hick-agent/crates/` since it does support the
agent's remote UI use case.

---

## 5. `hick-plan-runner` placement is acceptable as-is

**Location:** `hick-agent/crates/hick-plan-runner/`

`hick-plan-runner` executes hick plans using `hick-agent-sdk`. Unlike
`hick-query-edit`, it genuinely uses agent SDK machinery. Its current location
is correct.

---

## Workspace `members` after moves (Option A for items 1–3)

```toml
[workspace]
members = [
    "backend",
    "crates/*",                     # pipeline primitives + bench
    "examples/*",                   # wasm-test-tool, wasi-example
    "hick-lang/crates/*",           # hick-lang, hick-lsp, hick-lens, hick-mcp, hick-query-edit
    "hick-reactive/crates/*",       # hick-flow, hick-grove, hick-grove-server, hick-live
    "hick-exec/crates/*",           # hick-exec, hick-condition
    "hick-security/crates/*",
    "hick-store/crates/*",
    "wasm-net/crates/*",
    "hick-container/crates/*",      # + hick-api
    "hick-agent/crates/*",
    "native2wasm/crates/*",
]
```
