# Variables and Parameters

Support document-level variables and CLI-supplied parameters for parameterized pipelines.

## Motivation

Currently, all values in a `.hick` document are hardcoded. The copy/paste system allows reuse of inline values, but there's no way to externalize values — you can't run the same document with different configurations without editing it. RMarkdown has `params` in YAML front matter; Quarto has `{{< var >}}` from `_variables.yml`. Hickory Docs needs both internal variables and external parameters.

## Design

### Two Mechanisms

1. **`<hick:var>` — Document Variables**: Declared inline, referenced by name. Like `<hick:copy>` but lighter-weight (no clipboard semantics, just key-value substitution).

2. **`--param key=value` — CLI Parameters**: Passed at runtime, override document-level defaults. Enable parameterized pipelines without editing source.

### Tag: `<hick:var>`

```xml
<!-- Declare a variable with a default value -->
<hick:var name="version">3.12.0</hick:var>

<!-- Reference a variable (self-closing) -->
<hick:file path="out.txt">
Python version: <hick:val name="version" />
</hick:file>
```

### Tag: `<hick:val>`

Self-closing reference tag that inserts a variable's value. The `name` attribute specifies which variable to look up.

Why separate tags (`var` for declaration, `val` for reference) instead of overloading `paste`?

- `copy`/`paste` is a clipboard metaphor — you paste content from a named slot
- `var`/`val` is a variable metaphor — you declare a binding and reference it
- Variables can have defaults overridden by CLI params; copy blocks can't
- Variables don't appear in output (like `cut`); `copy` does appear in output
- Keeping them separate avoids confusion about which system applies

### CLI Parameters

```bash
# Override a variable
hick --param version=3.13.0 docs/guide.hick

# Multiple parameters
hick --param version=3.13.0 --param region=eu-west-1 docs/deploy.hick

# Parameters that aren't declared as variables produce a warning
```

### Resolution Order

1. CLI `--param` values (highest priority)
2. `<hick:var>` declarations in the document
3. Missing variable with no default → warning + empty string

### Scope

Variables are global across all documents in a pipeline run (same as copy/paste). A variable declared in one document is accessible from `<hick:val>` in another.

### Variables in Attributes

Variables can also be referenced in tag attributes using `{var:name}` interpolation:

```xml
<hick:var name="image-tag">3.20</hick:var>

<hick:container name="builder" image="alpine:{var:image-tag}">
  <hick:deny network="*" />
</hick:container>
```

This allows parameterizing container images, file paths, and other attributes.

## Implementation

### Step 1: Add Variable Storage to State

**File:** `crates/hick-exec/src/state.rs`

```rust
pub struct MultiDocumentState {
    // ... existing fields ...
    variables: Mutex<HashMap<String, String>>,
}

impl MultiDocumentState {
    pub fn register_var(&self, name: String, value: String) {
        let mut vars = self.variables.lock().unwrap();
        // Only set if not already set (CLI params take precedence)
        vars.entry(name).or_insert(value);
    }

    pub fn set_param(&self, name: String, value: String) {
        let mut vars = self.variables.lock().unwrap();
        // Always overwrite (CLI params override document defaults)
        vars.insert(name, value);
    }

    pub fn resolve_var(&self, name: &str) -> Option<String> {
        let vars = self.variables.lock().unwrap();
        vars.get(name).cloned()
    }
}
```

Note the asymmetry: `register_var` uses `entry().or_insert()` (doesn't overwrite), while `set_param` uses `insert()` (always overwrites). This means if CLI params are loaded first, document-level `<hick:var>` declarations don't override them.

### Step 2: Parse `--param` CLI Arguments

**File:** `crates/hick/src/main.rs`

```rust
#[derive(Parser)]
struct Cli {
    // ... existing fields ...

    /// Set a variable: --param key=value
    #[arg(long = "param", value_parser = parse_param)]
    params: Vec<(String, String)>,
}

fn parse_param(s: &str) -> Result<(String, String), String> {
    let (key, value) = s.split_once('=')
        .ok_or_else(|| format!("invalid param format '{s}', expected key=value"))?;
    Ok((key.to_string(), value.to_string()))
}
```

### Step 3: Pass Parameters into Pipeline

**File:** `crates/hick/src/lib.rs`

Add a params argument to the pipeline functions:

```rust
pub async fn run_pipeline(
    sources: &[(&str, &str)],
    params: &[(String, String)],
) -> Result<PipelineResult> {
    // ...
    run_pipeline_with_authority(sources, &authority, params).await
}

pub async fn run_pipeline_with_authority(
    sources: &[(&str, &str)],
    authority: &Arc<TokenAuthority>,
    params: &[(String, String)],
) -> Result<PipelineResult> {
    // ...

    let state = Arc::new(MultiDocumentState::default());

    // Load CLI params first (highest priority)
    for (key, value) in params {
        state.set_param(key.clone(), value.clone());
    }

    // Process var declarations (won't override CLI params)
    for node in &all_nodes {
        if let HickNode::Tag(tag) = node && tag.name == "var" {
            let name = tag_attr(tag, "name").unwrap_or_default();
            let value = collect_text_children(tag);
            state.register_var(name, value);
        }
    }

    // ... rest of pipeline ...
}
```

### Step 4: Resolve `<hick:val>` in File Outputs

**File:** `crates/hick/src/lib.rs`

In the file-output processing loop, add a branch for `val` tags:

```rust
HickNode::Tag(child_tag) if child_tag.name == "val" => {
    let var_name = tag_attr(child_tag, "name").unwrap_or_default();
    if let Some(value) = state.resolve_var(&var_name) {
        file_ip.add(Arc::new(StringNode::new(value)));
    } else {
        warn!("Variable '{var_name}' not found");
    }
}
```

This goes in both `run_pipeline_with_authority()` and `run_pipeline_live()`.

### Step 5: Attribute Interpolation

**File:** `crates/hick/src/lib.rs`

Add an interpolation function for attribute values:

```rust
fn interpolate_vars(value: &str, state: &MultiDocumentState) -> String {
    let mut result = String::new();
    let mut rest = value;
    while let Some(start) = rest.find("{var:") {
        result.push_str(&rest[..start]);
        let after_marker = &rest[start + 5..]; // skip "{var:"
        if let Some(end) = after_marker.find('}') {
            let var_name = &after_marker[..end];
            if let Some(val) = state.resolve_var(var_name) {
                result.push_str(&val);
            } else {
                warn!("Variable '{var_name}' not found in attribute interpolation");
                result.push_str(&rest[start..start + 5 + end + 1]); // keep original
            }
            rest = &after_marker[end + 1..];
        } else {
            result.push_str(&rest[start..]);
            break;
        }
    }
    result.push_str(rest);
    result
}
```

Use this when reading container image attributes and file path attributes:

```rust
// In container collection:
let image = tag_attr(tag, "image")
    .map(|v| interpolate_vars(&v, &state));

// In exec processing:
let image = exec_info.image.as_deref()
    .map(|v| interpolate_vars(v, &state))
    .unwrap_or_else(|| "alpine".to_string());
```

**Note on ordering:** Variable interpolation in attributes requires that variables are registered (Step 3) before container/exec processing. The current pipeline order already processes copy/cut before file outputs, so variables should be processed in the same early phase.

However, attribute interpolation for container images happens during container collection, which currently happens before copy/cut processing. We need to reorder:

1. Parse all documents
2. Build DAGs (validation only)
3. Load CLI params
4. Process `<hick:var>` declarations
5. Collect containers (with attribute interpolation)
6. Mint tokens
7. Process copy/cut
8. Execute (live) or build transcripts (dry-run)
9. Process file outputs (with `<hick:val>` resolution)

### Step 6: Tests

**File:** `crates/hick/tests/pipeline_tests.rs`

```rust
#[tokio::test]
async fn test_var_val_basic() {
    let src = hick_doc(concat!(
        r#"<hick:var name="version">1.0.0</hick:var>"#,
        "\n",
        r#"<hick:file path="out.txt">v=<hick:val name="version" /></hick:file>"#,
    ));
    let result = hick::run_pipeline(&[("test.hick", &src)], &[]).await.unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "v=1.0.0");
}

#[tokio::test]
async fn test_param_overrides_var() {
    let src = hick_doc(concat!(
        r#"<hick:var name="version">1.0.0</hick:var>"#,
        "\n",
        r#"<hick:file path="out.txt">v=<hick:val name="version" /></hick:file>"#,
    ));
    let params = vec![("version".to_string(), "2.0.0".to_string())];
    let result = hick::run_pipeline(&[("test.hick", &src)], &params).await.unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "v=2.0.0");
}

#[tokio::test]
async fn test_var_missing_warns() {
    let src = hick_doc(
        r#"<hick:file path="out.txt">before<hick:val name="missing" />after</hick:file>"#,
    );
    let result = hick::run_pipeline(&[("test.hick", &src)], &[]).await.unwrap();
    // Missing variables produce no output (like missing paste)
    assert_eq!(result.files.get("out.txt").unwrap(), "beforeafter");
}

#[tokio::test]
async fn test_var_across_documents() {
    let doc1 = hick_doc(r#"<hick:var name="shared">hello</hick:var>"#);
    let doc2 = hick_doc(
        r#"<hick:file path="out.txt"><hick:val name="shared" /></hick:file>"#,
    );
    let result = hick::run_pipeline(
        &[("doc1.hick", &doc1), ("doc2.hick", &doc2)], &[]
    ).await.unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "hello");
}
```

## Files Modified

| File | Change |
|------|--------|
| `crates/hick-exec/src/state.rs` | Add `variables` field, `register_var()`, `set_param()`, `resolve_var()` |
| `crates/hick/src/main.rs` | `--param key=value` CLI argument |
| `crates/hick/src/lib.rs` | Process `<hick:var>` tags, resolve `<hick:val>` in file outputs, `interpolate_vars()` for attributes, add `params` argument to pipeline functions |
| `crates/hick/tests/pipeline_tests.rs` | Tests for var/val, param override, cross-document, missing variables |

## Migration

- `run_pipeline()` signature changes to accept `params: &[(String, String)]`. All existing callers (tests, main) need updating. Tests pass `&[]` for no params.
- No changes to existing document syntax. New tags are additive.

## Relationship to Copy/Paste

`var`/`val` and `copy`/`paste` serve different purposes and coexist:

| Feature | `copy`/`paste` | `var`/`val` |
|---------|---------------|-------------|
| Declaration visibility | `copy` appears in output; `cut` is hidden | `var` is always hidden |
| Override from CLI | No | Yes (`--param`) |
| Content type | Arbitrary text blocks | Simple scalar values |
| Selector syntax | `select="#id"` | `name="key"` |
| Attribute interpolation | No | Yes (`{var:key}`) |
