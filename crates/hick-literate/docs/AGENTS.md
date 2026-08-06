# hick

Hick is a secure container orchestration shell for generating code and documentation. It processes `.hick` files that define containers, execute commands, and produce file outputs.

## Key Concepts

### .hick Files

Hick files are XML-like documents with the `hick:` namespace prefix:

```xml
<hick:doc>
    <hick:container name="build" image="alpine">
        <hick:allow network="api.example.com:443" />
    </hick:container>

    <hick:file path="output.txt">
        <hick:exec container="build">
            echo "Hello from container"
        </hick:exec>
    </hick:file>
</hick:doc>
```

### IMPORTANT: Do Not Escape XML Entities

**Never escape XML entities in `.hick` file content.** Hick has its own parser that preserves text content literally. Using XML escapes will produce incorrect output.

**WRONG** (do not do this):
```xml
<hick:file path="example.swift">
    if x &lt; 10 &amp;&amp; y &gt; 5 {
        print(&quot;hello&quot;)
    }
</hick:file>
```

**CORRECT** (use raw characters):
```xml
<hick:file path="example.swift">
    if x < 10 && y > 5 {
        print("hello")
    }
</hick:file>
```

This applies to all content inside hick tags:
- Use `<` not `&lt;`
- Use `>` not `&gt;`
- Use `&` not `&amp;`
- Use `"` not `&quot;`
- Use `'` not `&apos;`

The only XML syntax that matters is the `<hick:...>` tags themselves. Everything else is literal text.

### Containers

Containers are isolated WASM environments with capability-based security:

- `<hick:container name="..." image="...">` — Define a container
- `<hick:allow network="host:port" />` — Grant network access
- `<hick:allow file-read="path" />` — Grant file read access
- `<hick:allow file-write="path" />` — Grant file write access
- `<hick:deny network="*" />` — Deny all network access
- `<hick:secret name="ENV_VAR" from="secret-name" />` — Inject a secret

### Exec Blocks

Execute commands in containers and capture output:

```xml
<hick:exec container="container-name">
    commands here
</hick:exec>
```

The `show` attribute controls output rendering:
- `show="all"` (default) — Show commands and output
- `show="command"` — Show only the command
- `show="output"` — Show only the output
- `show="none"` — Side-effect only, render nothing

### File Outputs

Define output files with `<hick:file path="...">`:

```xml
<hick:file path="src/{{project_name}}/App.swift">
    Content here with {{variable}} interpolation
</hick:file>
```

Path interpolation supports case variants: `{{name:snake_case}}`, `{{name:PascalCase}}`, etc.

### Variables and Substitution

```xml
<hick:var name="project_name">MyApp</hick:var>
<hick:val name="project_name" />

<hick:substitute name="project" pattern="TEMPLATE" value="{{project_name}}" variants="true" />
```

The `variants="true"` attribute generates all case variants (PascalCase, snake_case, etc.).

### Copy/Cut/Paste

Share content between files:

```xml
<hick:copy id="shared-code" class="utilities">
    shared content here
</hick:copy>

<hick:paste select="#shared-code" />
<hick:paste select=".utilities" />
```

### Conditional Content

Include content based on conditions:

```xml
<hick:when test="feature_enabled">
    conditional content
</hick:when>

<hick:file path="optional.txt" when="include_optional">
    only included if include_optional is set
</hick:file>
```

### Features

Define and use feature flags:

```xml
<hick:feature name="auth" description="Authentication support" />
<hick:feature name="oauth" description="OAuth provider" requires="auth" />

<hick:when test="auth">
    Auth-specific content
</hick:when>
```

Enable features via CLI: `hick --features auth,oauth`

#### Feature Mutual Exclusivity

Use `conflicts_with` for arbitrary pairwise conflicts:

```xml
<hick:feature name="oidc" description="OIDC auth" conflicts_with="p2p-auth" />
<hick:feature name="p2p-auth" description="P2P auth" conflicts_with="oidc" />
```

Use `exclusive_group` for "pick one of N" scenarios:

```xml
<hick:feature name="postgres" description="PostgreSQL" exclusive_group="database" />
<hick:feature name="sqlite" description="SQLite" exclusive_group="database" />
<hick:feature name="mysql" description="MySQL" exclusive_group="database" />
```

Features in the same exclusive group or with explicit conflicts cannot be enabled together. The `generate-matrix` command automatically filters out invalid combinations.

### Volumes

Share data between containers:

```xml
<hick:volume name="shared" input="./input" output="./output" />

<hick:exec container="producer" mount="shared:/data">
    produce files to /data
</hick:exec>

<hick:exec container="consumer" mount="shared:/data">
    consume files from /data
</hick:exec>
```

### Weave (Literate Programming)

Generate documentation alongside code:

```xml
<hick:doc weave="docs/architecture.md">
    Prose content becomes documentation.

    <hick:file path="src/main.rs" doc-hidden="false">
        // This appears in a fenced code block
    </hick:file>
</hick:doc>
```

### Includes

Split large documents into modules:

```xml
<hick:include path="./components/auth.hick" />
```

## IMPORTANT: Always Verify with generate-matrix

**After making ANY changes to `.hick` files, you MUST run `hick generate-matrix` to verify all feature combinations work correctly.**

```bash
# Basic verification - generates all combinations and checks they parse
hick generate-matrix ./path/to/hick/files/

# With custom verification command (recommended)
hick generate-matrix --verify "cargo check" ./path/to/hick/files/

# Continue checking all combinations even if some fail
hick generate-matrix --verify "npm run build" --continue-on-error ./project/

# Run verifications in parallel for speed
hick generate-matrix -j 8 --verify "make test" ./templates/
```

### Why This Matters

A `.hick` file may work fine with one feature combination but fail with others. The `generate-matrix` command:

1. Generates all valid feature combinations (respecting `conflicts_with` and `exclusive_group`)
2. Produces output for each combination in separate directories
3. Optionally runs a verification command against each generated output

### Custom Verify Commands

The `--verify` flag accepts any shell command. You can specify multiple `--verify` flags, and each command runs in sequence. Use template variables for dynamic paths:

- `{{features}}` — comma-separated list of enabled features
- `{{dir}}` — path to the generated output directory

Examples:

```bash
# Check Rust code compiles
hick generate-matrix --verify "cargo check --manifest-path {{dir}}/Cargo.toml" ./

# Run TypeScript type checking
hick generate-matrix --verify "cd {{dir}} && npx tsc --noEmit" ./

# Validate JSON/YAML configs
hick generate-matrix --verify "cd {{dir}} && yamllint ." ./

# Custom script
hick generate-matrix --verify "./scripts/validate.sh {{dir}} {{features}}" ./

# Multiple verify commands (all run in sequence)
hick generate-matrix \
  --verify "cargo check" \
  --verify "cargo test" \
  --verify "cargo clippy" ./
```

### Conditional Verify Commands

Prefix a verify command with `[feature]` to run it only when specific features are enabled. This is useful when different feature combinations require different verification steps.

**Syntax:**

- `--verify "command"` — Always runs
- `--verify "[feature] command"` — Runs only if `feature` is enabled
- `--verify "[f1,f2] command"` or `--verify "[f1|f2] command"` — Runs if ANY of these features enabled (OR logic)
- `--verify "[f1+f2] command"` — Runs if ALL of these features enabled (AND logic)

**Examples:**

```bash
# Always echo the features
hick generate-matrix --verify "echo 'Generated: {{features}}'" ./

# Only run cargo check when rust-frontend-lib is enabled
hick generate-matrix --verify "[rust-frontend-lib] cargo check" ./

# Run npm test if spa OR extension is enabled
hick generate-matrix --verify "[spa,extension] npm test" ./

# Run e2e tests only when BOTH spa AND backend are enabled
hick generate-matrix --verify "[spa+backend] npm run e2e" ./

# Full example with multiple conditional verifies
hick generate-matrix . \
  --verify "echo 'Checking {{features}}'" \
  --verify "[rust-frontend-lib] cargo check" \
  --verify "[spa,extension] npm run lint" \
  --verify "[spa,extension] npm test" \
  --verify "[backend] cargo test --package backend" \
  --verify "[spa+backend] npm run e2e"
```

This allows you to define feature-specific verification without running irrelevant checks for combinations that don't include those features.

### Recommended Workflow

1. Make changes to `.hick` files
2. Run `hick generate-matrix --verify "your-check-command" ./`
3. Fix any failures
4. Commit only after all combinations pass

## Common Commands

```bash
# Run pipeline once
hick run file.hick

# Run all .hick files in directory
hick run ./project/

# Run with variables
hick run --param project_name=MyApp file.hick

# Run with features enabled
hick run --features auth,oauth file.hick

# Watch mode (re-run on changes)
hick up file.hick

# Interactive REPL
hick

# With caching
hick run --cache file.hick

# Freeze mode (require all results cached)
hick run --freeze file.hick

# Dry run (placeholder output)
hick run --dry-run file.hick
```

## Project Configuration (_hick.yml)

```yaml
files:
  - "*.hick"
  - "templates/**/*.hick"

vars:
  project_name: "MyProject"
  version: "1.0.0"

output-dir: "./generated"

secrets:
  key-file: "~/.config/hick/key.txt"
  secrets-dir: "~/.config/hick/secrets"

defaults:
  images-dir: "./images"
```

## File Structure

Typical project layout:

```
project/
├── _hick.yml           # Project config
├── main.hick           # Main document
├── templates/          # Template .hick files
│   ├── ios.hick
│   └── android.hick
├── generated/          # Output directory
└── .hick-cache/        # Execution cache
```

## MCP Server (hick-mcp)

For deeper integration, the `hick-mcp` binary exposes hick operations as MCP
tools over stdio. This enables AI coding agents to:

- **Run pipelines** (`hick_run`) and cache results
- **Query output structure** (`hick_structure`) — list functions, structs, classes with line ranges
- **Search by selector** (`hick_query`) — e.g. `fn main`, `struct Config`, `.imports`
- **Inspect provenance** (`hick_provenance`) — see which `.hick` source positions produced a given output range
- **Preview output** (`hick_preview`) — get full rendered content of any output file
- **Compute edits** (`hick_replace`) — determine which `.hick` source byte ranges to edit when modifying a structural element

Add to your MCP configuration:

```json
{
  "mcpServers": {
    "hick": {
      "command": "hick-mcp"
    }
  }
}
```

### Sandboxed Command Execution (hick_exec)

When started with `--images-dir`, hick-mcp also provides the `hick_exec` tool
for running arbitrary shell commands inside WASM Linux containers:

```json
{"tool": "hick_exec", "arguments": {"command": "cargo test", "timeout": 300}}
```

The host working directory is mounted at `/workspace` in the container.
Containers persist across calls, so installed packages and environment state
are preserved within a session. Use the `container` parameter to manage
multiple isolated environments.

See [Claude Code Setup](../../../docs/claude-code-setup.md) for instructions on
configuring Claude Code to use sandboxed execution exclusively.

See the [MCP Reference](../../../docs/mcp-reference.md) for full tool documentation.
