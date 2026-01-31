# File Includes

Allow one `.hick` document to include another at a specific insertion point, enabling modular document composition without passing every file on the CLI.

## Motivation

Currently, multi-document support requires listing all files on the command line (`hick a.hick b.hick c.hick`). Documents share copy/paste state but there's no way for one document to pull another's content into a specific location. For larger projects (books, multi-section guides), authors need a way to compose documents from reusable fragments.

## Design

### New Tag: `<hick:include>`

```xml
<h:doc xmlns:h="http://www.hickorydocs.com/1.0">

<!-- Include a shared header fragment -->
<h:include file="fragments/header.hick" />

<!-- Main content -->
<h:file path="guide.md">
# Chapter 1

<h:include file="fragments/common-setup.hick" />

Chapter-specific content here.
</h:file>

</h:doc>
```

### Semantics

- `<hick:include file="path" />` is a self-closing tag
- The `file` attribute is a relative path, resolved from the directory of the including document
- Include processing happens **before** DAG building and pipeline execution
- The included file's content is spliced into the parent document's AST at the include site
- Included files are themselves full `.hick` documents (with `<hick:doc>` root) — only the root's children are spliced in
- Includes can be nested (an included file can include other files)
- Circular includes are detected and rejected with a clear error

### Include vs CLI Multi-Document

| Mechanism | Use case |
|-----------|----------|
| CLI args (`hick a.hick b.hick`) | Independent documents processed in same pipeline, sharing copy/paste state |
| `<hick:include file="..." />` | Composition — fragment inserted at a specific point in the parent |

Both can be used together. CLI-listed documents are processed independently (each produces its own file outputs). Includes are resolved within a single document before processing.

### What Can Be Included

Included files can contain any hick tags:
- Container definitions (shared across the document)
- Exec tags (added to the DAG)
- Copy/cut blocks (available for paste in the parent)
- File output fragments (nested inside the parent's file tag)
- Other includes (recursive)

### What Cannot Be Included

- Non-`.hick` files (plain text, markdown). Use copy/paste or exec output for that.
- Files outside the project directory (path traversal with `../` above the root document's directory is rejected for security)

## Implementation

### Step 1: Add Include Resolution Pass

**File:** `crates/hick-lang/src/lib.rs`

Add a post-parse include resolution function. This operates on the parsed AST, finding `<hick:include>` tags and replacing them with the included document's children.

```rust
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Resolve all `<hick:include>` tags in a parsed document, recursively.
///
/// `base_dir` is the directory of the document being processed.
/// `seen` tracks already-included paths for cycle detection.
pub fn resolve_includes(
    doc: &mut HickDocument,
    base_dir: &Path,
    seen: &mut HashSet<PathBuf>,
) -> Result<(), ParseError> {
    let mut new_nodes = Vec::new();
    for node in std::mem::take(&mut doc.nodes) {
        match &node {
            HickNode::Tag(tag) if tag.name == "include" => {
                let file_attr = tag.get_attribute("file")
                    .ok_or_else(|| ParseError::Syntax {
                        line: tag.source_line,
                        message: "<hick:include> missing 'file' attribute".into(),
                    })?;

                let include_path = base_dir.join(file_attr);
                let canonical = include_path.canonicalize().map_err(|e| {
                    ParseError::Syntax {
                        line: tag.source_line,
                        message: format!("include path '{}': {e}", include_path.display()),
                    }
                })?;

                // Cycle detection
                if !seen.insert(canonical.clone()) {
                    return Err(ParseError::Syntax {
                        line: tag.source_line,
                        message: format!("circular include: {}", canonical.display()),
                    });
                }

                // Parse included file
                let source = std::fs::read_to_string(&canonical).map_err(|e| {
                    ParseError::Syntax {
                        line: tag.source_line,
                        message: format!("cannot read '{}': {e}", canonical.display()),
                    }
                })?;
                let mut included_doc = parse(&source)?;

                // Recursively resolve includes in the included doc
                let included_dir = canonical.parent().unwrap_or(base_dir);
                resolve_includes(&mut included_doc, included_dir, seen)?;

                // Splice children into parent
                new_nodes.extend(included_doc.nodes);

                seen.remove(&canonical);
            }
            HickNode::Tag(tag) => {
                // Recurse into children of other tags (e.g., file tags)
                let mut mutable_tag = tag.clone();
                resolve_includes_in_children(&mut mutable_tag.children, base_dir, seen)?;
                new_nodes.push(HickNode::Tag(mutable_tag));
            }
            _ => {
                new_nodes.push(node);
            }
        }
    }
    doc.nodes = new_nodes;
    Ok(())
}

fn resolve_includes_in_children(
    children: &mut Vec<HickNode>,
    base_dir: &Path,
    seen: &mut HashSet<PathBuf>,
) -> Result<(), ParseError> {
    let mut new_children = Vec::new();
    for child in std::mem::take(children) {
        match &child {
            HickNode::Tag(tag) if tag.name == "include" => {
                // Same logic as above — extract to shared function
                // ...
            }
            HickNode::Tag(tag) => {
                let mut mutable_tag = tag.clone();
                resolve_includes_in_children(&mut mutable_tag.children, base_dir, seen)?;
                new_children.push(HickNode::Tag(mutable_tag));
            }
            _ => {
                new_children.push(child);
            }
        }
    }
    *children = new_children;
    Ok(())
}
```

### Step 2: Integrate into Pipeline

**File:** `crates/hick/src/lib.rs`

After parsing each document, resolve includes before DAG building:

```rust
// Parse all sources
let mut documents = Vec::new();
for (name, source) in sources {
    let mut doc = hick_lang::parse(source)?;

    // Resolve includes relative to the source file's directory
    let base_dir = Path::new(name).parent().unwrap_or(Path::new("."));
    let mut seen = HashSet::new();
    if let Some(canonical) = std::fs::canonicalize(name).ok() {
        seen.insert(canonical);
    }
    hick_lang::resolve_includes(&mut doc, base_dir, &mut seen)?;

    documents.push((*name, doc));
}
```

This goes in both `run_pipeline_with_authority()` and `run_pipeline_live()`.

### Step 3: Path Security

**File:** `crates/hick-lang/src/lib.rs`

Before resolving the include path, validate it doesn't escape the project root:

```rust
fn validate_include_path(include_path: &Path, base_dir: &Path) -> Result<(), String> {
    let canonical = include_path.canonicalize()
        .map_err(|e| format!("cannot resolve: {e}"))?;
    let base_canonical = base_dir.canonicalize()
        .unwrap_or_else(|_| base_dir.to_path_buf());

    // The included file must be under the base directory or a sibling
    // For now, just reject paths containing ".." that escape
    if !canonical.starts_with(&base_canonical) {
        return Err(format!(
            "include path {} is outside project root {}",
            canonical.display(),
            base_canonical.display(),
        ));
    }
    Ok(())
}
```

Note: The exact security boundary depends on the project configuration feature. For now, use the directory of the root `.hick` file as the boundary. When project configuration lands, use the project root.

### Step 4: Update Parser to Recognize `include` Tag

The parser already handles any `<hick:TAG_NAME>` — it will parse `<hick:include file="..." />` as a self-closing tag with a `file` attribute. No parser changes needed; the `include` tag name is just another tag as far as the parser is concerned. Resolution happens post-parse.

### Step 5: Tests

**File:** `crates/hick/tests/pipeline_tests.rs`

```rust
#[tokio::test]
async fn test_include_basic() {
    // This test needs files on disk; use a temp directory
    let dir = std::env::temp_dir().join("hick-test-include");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // Write fragment
    std::fs::write(dir.join("version.hick"), r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:copy id="ver">4.0.0</hick:copy>
</hick:doc>"#).unwrap();

    // Write main doc
    let main_src = format!(r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:include file="version.hick" />
<hick:file path="out.txt">v=<hick:paste select="#ver" /></hick:file>
</hick:doc>"##);
    let main_path = dir.join("main.hick");
    std::fs::write(&main_path, &main_src).unwrap();

    let result = hick::run_pipeline(&[(main_path.to_str().unwrap(), &main_src)])
        .await.unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "v=4.0.0");

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn test_include_circular_detected() {
    let dir = std::env::temp_dir().join("hick-test-include-cycle");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // a.hick includes b.hick, b.hick includes a.hick
    std::fs::write(dir.join("a.hick"), r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:include file="b.hick" />
</hick:doc>"#).unwrap();

    std::fs::write(dir.join("b.hick"), r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:include file="a.hick" />
</hick:doc>"#).unwrap();

    let src = std::fs::read_to_string(dir.join("a.hick")).unwrap();
    let result = hick::run_pipeline(&[(dir.join("a.hick").to_str().unwrap(), &src)]).await;
    assert!(result.is_err(), "circular include should fail");

    let _ = std::fs::remove_dir_all(&dir);
}
```

**File:** `crates/hick-lang/src/lib.rs` (unit tests)

```rust
#[test]
fn parse_include_tag() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:include file="other.hick" />
</hick:doc>"#;
    let doc = parse(src).unwrap();
    assert_eq!(doc.nodes.len(), 1);
    if let HickNode::Tag(tag) = &doc.nodes[0] {
        assert_eq!(tag.name, "include");
        assert!(tag.self_closing);
        assert_eq!(tag.get_attribute("file").unwrap(), "other.hick");
    } else {
        panic!("expected include tag");
    }
}
```

## Files Modified

| File | Change |
|------|--------|
| `crates/hick-lang/src/lib.rs` | `resolve_includes()`, `resolve_includes_in_children()`, `validate_include_path()`, tests |
| `crates/hick/src/lib.rs` | Call `resolve_includes()` after parsing in both pipeline functions |
| `crates/hick/tests/pipeline_tests.rs` | Integration tests with temp files |

## Design Decisions

- **Post-parse resolution** (not during parse): Keeps the parser simple and stateless. The parser doesn't need filesystem access.
- **Full documents as includes**: Included files are valid `.hick` documents, not fragments. This means they can be tested independently.
- **Cycle detection via path set**: Simple and reliable. The set tracks canonical paths to handle symlinks.
- **Path security**: Includes are restricted to the project subtree. This aligns with the security-first philosophy.
