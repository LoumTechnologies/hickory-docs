# Conditional Content

Skip or include document sections based on variables, parameters, or execution context.

## Motivation

Currently every tag in a `.hick` document is unconditional — all execs run, all file sections are included, all containers are started. Authors need:

- Environment-specific content (dev vs prod deployment steps)
- Optional sections toggled by a parameter
- Skip expensive execs during documentation-only builds
- Platform-specific instructions in generated docs

Quarto uses `.content-visible when-format="html"` and `when-profile="advanced"`. RMarkdown uses R expressions in chunk options (`eval=params$detailed`). Hickory Docs needs a mechanism that works with its XML tag model and integrates with the variables/parameters system.

## Design

### Approach: `when` Attribute on Any Tag

Rather than introducing a dedicated conditional tag, add a `when` attribute that can appear on any hick tag. If the condition evaluates to false, the tag (and all its children) is skipped entirely.

```xml
<hick:var name="env">dev</hick:var>

<!-- Only define prod container when env=prod -->
<hick:container name="deployer" image="alpine" when="env=prod">
  <hick:allow network="registry.example.com:443" />
  <hick:deny network="*" />
</hick:container>

<!-- Only run deployment when env=prod -->
<hick:exec container="deployer" when="env=prod">
./deploy.sh
</hick:exec>

<!-- Conditional file section -->
<hick:file path="guide.md">
# Setup Guide

<hick:val name="env" /> environment instructions.

<!-- Include debug section only in dev -->
<hick:when test="env=dev">
## Debug Mode

Enable debug logging with `LOG_LEVEL=debug`.
</hick:when>
</hick:file>
```

### Condition Syntax

Simple equality and presence checks — no expression language. Conditions reference variables/parameters.

| Syntax | Meaning |
|--------|---------|
| `when="var_name"` | True if variable is defined and non-empty |
| `when="var_name=value"` | True if variable equals value |
| `when="var_name!=value"` | True if variable does not equal value |
| `when="!var_name"` | True if variable is undefined or empty |

Compound conditions (AND/OR) are deliberately excluded from v1. If needed, authors can use multiple nested conditional blocks or compute a derived variable in an exec.

### Two Mechanisms

1. **`when` attribute on existing tags** — Conditionally include/skip a tag:
   ```xml
   <hick:exec container="demo" when="env=prod">deploy</hick:exec>
   ```

2. **`<hick:when>` wrapper tag** — Conditionally include a block of children (useful inside `<hick:file>` for prose sections):
   ```xml
   <hick:when test="verbose">
   Detailed explanation here.
   </hick:when>
   ```

The `when` attribute is syntactic sugar for wrapping a single tag in `<hick:when>`. They are semantically identical.

### Interaction with DAG

Conditional execs that are skipped are excluded from the DAG entirely. This means:

- A skipped exec doesn't create container-state dependencies
- Volume dependencies from skipped execs don't exist
- Downstream execs that depended on a skipped exec become roots (or depend on the preceding non-skipped exec)

This is evaluated **after** variable/parameter resolution but **before** DAG construction.

### Interaction with File Outputs

Inside `<hick:file>`, `<hick:when test="...">` controls whether its children (text nodes, exec refs, paste refs) are included in the file output. Skipped sections produce no output.

## Implementation

### Step 1: Condition Evaluator

**New file:** `crates/hick/src/condition.rs`

```rust
use hick_exec::state::MultiDocumentState;

#[derive(Debug, Clone)]
pub enum Condition {
    /// Variable is defined and non-empty
    Defined(String),
    /// Variable is undefined or empty
    NotDefined(String),
    /// Variable equals value
    Equals(String, String),
    /// Variable does not equal value
    NotEquals(String, String),
}

impl Condition {
    /// Parse a condition string.
    pub fn parse(s: &str) -> Self {
        let s = s.trim();
        if let Some(rest) = s.strip_prefix('!') {
            Condition::NotDefined(rest.to_string())
        } else if let Some((name, value)) = s.split_once("!=") {
            Condition::NotEquals(name.to_string(), value.to_string())
        } else if let Some((name, value)) = s.split_once('=') {
            Condition::Equals(name.to_string(), value.to_string())
        } else {
            Condition::Defined(s.to_string())
        }
    }

    /// Evaluate against resolved variables.
    pub fn evaluate(&self, state: &MultiDocumentState) -> bool {
        match self {
            Condition::Defined(name) => {
                state.resolve_var(name).is_some_and(|v| !v.is_empty())
            }
            Condition::NotDefined(name) => {
                state.resolve_var(name).is_none_or(|v| v.is_empty())
            }
            Condition::Equals(name, expected) => {
                state.resolve_var(name).as_deref() == Some(expected.as_str())
            }
            Condition::NotEquals(name, expected) => {
                state.resolve_var(name).as_deref() != Some(expected.as_str())
            }
        }
    }
}
```

### Step 2: Conditional Filtering Pass

**File:** `crates/hick/src/lib.rs`

Add a filtering pass that removes nodes with false `when` conditions. This runs **after** variable/parameter loading and **before** DAG construction.

```rust
mod condition;

/// Remove nodes whose `when` condition evaluates to false.
fn filter_conditionals(nodes: &mut Vec<HickNode>, state: &MultiDocumentState) {
    nodes.retain_mut(|node| {
        match node {
            HickNode::Tag(tag) => {
                // Check `when` attribute on this tag
                if let Some(when_val) = tag_attr(tag, "when") {
                    let cond = Condition::parse(&when_val);
                    if !cond.evaluate(state) {
                        return false; // Remove this node
                    }
                }
                // Recurse into children
                filter_conditionals(&mut tag.children, state);
                true
            }
            HickNode::Text(_) => true,
        }
    });
}
```

Note: `<hick:when test="...">` tags are handled as a special case. When the condition is true, the `when` tag is replaced by its children (unwrapped). When false, the entire tag is removed:

```rust
fn filter_conditionals(nodes: &mut Vec<HickNode>, state: &MultiDocumentState) {
    let mut new_nodes = Vec::new();
    for node in std::mem::take(nodes) {
        match node {
            HickNode::Tag(mut tag) if tag.name == "when" => {
                let test = tag_attr(&tag, "test").unwrap_or_default();
                let cond = Condition::parse(&test);
                if cond.evaluate(state) {
                    // Condition true — unwrap children into parent
                    filter_conditionals(&mut tag.children, state);
                    new_nodes.extend(tag.children);
                }
                // Condition false — drop entire node
            }
            HickNode::Tag(mut tag) => {
                if let Some(when_val) = tag_attr(&tag, "when") {
                    let cond = Condition::parse(&when_val);
                    if !cond.evaluate(state) {
                        continue; // Skip this node
                    }
                }
                filter_conditionals(&mut tag.children, state);
                new_nodes.push(HickNode::Tag(tag));
            }
            text => {
                new_nodes.push(text);
            }
        }
    }
    *nodes = new_nodes;
}
```

### Step 3: Integrate into Pipeline Order

**File:** `crates/hick/src/lib.rs`

The pipeline order becomes:

```rust
// 1. Parse all documents
// 2. Load CLI params (set_param)
// 3. Process <hick:var> declarations (register_var)
// 4. Filter conditionals (removes nodes with false conditions)
// 5. Build and validate DAGs
// 6. Collect containers
// 7. Mint tokens
// 8. Process copy/cut
// 9. Execute or build transcripts
// 10. Process file outputs
```

Crucially, step 4 happens before step 5. This means skipped execs never enter the DAG, so there are no dangling dependencies.

```rust
// After variable loading...

// Filter conditionals on all documents
for (_, doc) in &mut documents {
    filter_conditionals(&mut doc.nodes, &state);
}

// Rebuild all_nodes after filtering
let all_nodes: Vec<&HickNode> = documents
    .iter()
    .flat_map(|(_, doc)| doc.nodes.iter())
    .collect();

// Now build DAGs, collect containers, etc.
```

### Step 4: Parser Changes

No parser changes needed. The parser already handles:
- `when="..."` as a regular attribute on any tag
- `<hick:when test="...">` as a regular tag with children

### Step 5: Tests

**File:** `crates/hick/tests/pipeline_tests.rs`

```rust
#[tokio::test]
async fn test_when_attribute_true() {
    let src = hick_doc(concat!(
        r#"<hick:var name="mode">full</hick:var>"#,
        "\n",
        r#"<hick:file path="out.txt">"#,
        r#"<hick:when test="mode=full">detailed content</hick:when>"#,
        r#"</hick:file>"#,
    ));
    let result = hick::run_pipeline(&[("test.hick", &src)], &[]).await.unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "detailed content");
}

#[tokio::test]
async fn test_when_attribute_false() {
    let src = hick_doc(concat!(
        r#"<hick:var name="mode">brief</hick:var>"#,
        "\n",
        r#"<hick:file path="out.txt">before"#,
        r#"<hick:when test="mode=full">detailed content</hick:when>"#,
        r#"after</hick:file>"#,
    ));
    let result = hick::run_pipeline(&[("test.hick", &src)], &[]).await.unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "beforeafter");
}

#[tokio::test]
async fn test_when_on_exec_skips_dag() {
    let src = hick_doc(concat!(
        r#"<hick:var name="run">no</hick:var>"#,
        "\n",
        r#"<hick:container name="demo" image="alpine" />"#,
        "\n",
        r#"<hick:exec container="demo" when="run=yes">echo hi</hick:exec>"#,
        "\n",
        r#"<hick:file path="out.txt">done</hick:file>"#,
    ));
    let result = hick::run_pipeline(&[("test.hick", &src)], &[]).await.unwrap();
    // Exec was skipped, but file output still works
    assert_eq!(result.files.get("out.txt").unwrap(), "done");
}

#[tokio::test]
async fn test_when_param_override() {
    let src = hick_doc(concat!(
        r#"<hick:var name="env">dev</hick:var>"#,
        "\n",
        r#"<hick:file path="out.txt">"#,
        r#"<hick:when test="env=prod">PRODUCTION</hick:when>"#,
        r#"<hick:when test="env=dev">DEVELOPMENT</hick:when>"#,
        r#"</hick:file>"#,
    ));
    // With default (dev)
    let result = hick::run_pipeline(&[("test.hick", &src)], &[]).await.unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "DEVELOPMENT");

    // With param override (prod)
    let params = vec![("env".into(), "prod".into())];
    let result = hick::run_pipeline(&[("test.hick", &src)], &params).await.unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "PRODUCTION");
}

#[tokio::test]
async fn test_when_defined_check() {
    let src = hick_doc(concat!(
        r#"<hick:file path="out.txt">"#,
        r#"<hick:when test="debug">debug mode</hick:when>"#,
        r#"<hick:when test="!debug">normal mode</hick:when>"#,
        r#"</hick:file>"#,
    ));
    // Without debug variable
    let result = hick::run_pipeline(&[("test.hick", &src)], &[]).await.unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "normal mode");

    // With debug variable
    let params = vec![("debug".into(), "1".into())];
    let result = hick::run_pipeline(&[("test.hick", &src)], &params).await.unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "debug mode");
}
```

## Files Modified

| File | Change |
|------|--------|
| `crates/hick/src/condition.rs` | New file: `Condition` enum, parse/evaluate |
| `crates/hick/src/lib.rs` | `filter_conditionals()`, pipeline reordering |
| `crates/hick/tests/pipeline_tests.rs` | Tests for when/conditional behavior |

## Dependencies

This feature depends on **Variables/Parameters** (the `var`/`val` system). Conditions reference variables by name, and CLI params enable runtime toggling. These should be implemented together or variables first.

## Design Decisions

- **No expression language**: Equality checks and presence checks cover the common cases. A full expression evaluator would add complexity without proportional value. If authors need complex logic, they can compute a derived variable in an exec step.
- **Attribute-based, not tag-based**: `when` on existing tags is less verbose than wrapping every conditional in a `<hick:if>` block. The `<hick:when>` wrapper is available for multi-node sections.
- **Pre-DAG evaluation**: Conditions are resolved before DAG construction. This means the DAG only contains nodes that will actually execute. The alternative (building the full DAG and skipping at execution time) would require handling dangling dependencies.
- **No else/else-if**: Use separate `<hick:when>` blocks with complementary conditions. This is more explicit and avoids ambiguity in XML structure.
