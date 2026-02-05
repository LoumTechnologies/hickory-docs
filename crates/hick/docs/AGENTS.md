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
