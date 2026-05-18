---
id: TASK-24.7
title: MCP response format config option for hick agent implementations
status: Done
assignee: []
created_date: '2026-05-16 12:56'
updated_date: '2026-05-18 01:00'
labels:
  - mcp
  - graphql
  - eval
  - context-efficiency
milestone: m-3
dependencies:
  - TASK-24.1
  - TASK-23
  - TASK-28
parent_task_id: TASK-24
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add a `mcp_response_format` configuration option to `hick-agent-sdk` so each agent implementation can control how MCP tool results are serialized into the conversation. Format can be set globally or per response shape — lists, scalars, nested objects, and flat objects can each use a different format. All four tool-calling modes (TASK-24.3–24.6) call a shared helper that detects the shape and applies the right format.

## Response shapes

```rust
pub enum ResponseShape {
    Scalar,   // primitive or single-field object with a primitive value
    Object,   // flat key-value object (depth ≤ 1)
    Nested,   // object with sub-objects (depth > 1)
    List,     // top-level array, or object whose primary field is an array
}
```

`detect_shape(value: &serde_json::Value) -> ResponseShape` inspects the value and returns the appropriate variant.

## Config

```toml
[mcp.response_format]
default = "json"   # fallback when no shape-specific rule matches

[mcp.response_format.by_shape]
list   = "xml"       # <items><item>...</item></items> — clear boundaries for LLM counting
scalar = "json"      # {"value": 42} — minimal noise
object = "json"      # flat objects stay as JSON
nested = "markdown"  # nested objects as nested headers/bullets
```

In Rust:

```rust
pub enum McpResponseFormat { Json, Markdown, Xml }

pub struct ShapeFormatConfig {
    pub default: McpResponseFormat,
    pub list:    Option<McpResponseFormat>,
    pub scalar:  Option<McpResponseFormat>,
    pub object:  Option<McpResponseFormat>,
    pub nested:  Option<McpResponseFormat>,
}

impl ShapeFormatConfig {
    pub fn format_for(&self, shape: ResponseShape) -> &McpResponseFormat {
        match shape {
            ResponseShape::List   => self.list.as_ref(),
            ResponseShape::Scalar => self.scalar.as_ref(),
            ResponseShape::Object => self.object.as_ref(),
            ResponseShape::Nested => self.nested.as_ref(),
        }
        .unwrap_or(&self.default)
    }
}
```

The shared helper becomes:

```rust
pub fn format_mcp_result(value: &serde_json::Value, config: &ShapeFormatConfig) -> String {
    let shape = detect_shape(value);
    let format = config.format_for(shape);
    serialize(value, format)
}
```

## Format examples by shape

**List → XML:**
```xml
<hick:tool-result>
  <items>
    <item><number>42</number><url>https://github.com/...</url></item>
    <item><number>43</number><url>https://github.com/...</url></item>
  </items>
</hick:tool-result>
```

**Scalar → JSON:**
```json
{"exit_code": 0}
```

**Flat object → JSON:**
```json
{"number": 42, "url": "https://github.com/..."}
```

**Nested → Markdown:**
```markdown
### githubGetPR
- **title**: Fix bug
- **author**: login: nate, url: https://github.com/nate
- **labels**: [bug, high-priority]
```

## Defaults

The defaults above (`list=xml`, `scalar=json`, `object=json`, `nested=markdown`) are the recommended starting point, informed by the eval. A single `default = "json"` with no `by_shape` table is also valid and produces uniform JSON output.

## Eval to inform defaults

Run all three formats × four shapes through the TASK-23 eval framework. The results set the recommended per-shape defaults documented in-repo. The config ships regardless.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 ResponseShape enum and detect_shape function are defined in hick-agent-sdk and correctly classify scalars, flat objects, nested objects, and lists.
- [ ] #2 ShapeFormatConfig is the config type with a default format and optional per-shape overrides; it deserializes correctly from the TOML schema above.
- [ ] #3 format_mcp_result(value, config) detects shape, selects format, and serializes correctly for all four shape/format combinations.
- [ ] #4 All four extension crates use format_mcp_result — no inline format logic per extension.
- [ ] #5 A config with only default set (no by_shape table) produces uniform output in that format.
- [ ] #6 Each agent binary (copper-ledge, iron-peak) can set its own ShapeFormatConfig without code changes.
- [ ] #7 Eval covers all three formats across all four shapes (12 variants) through the TASK-23 framework; results documented in-repo.
- [ ] #8 Recommended per-shape defaults are set from eval results (or justified fallbacks) and documented in the config TOML example.
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented hick-agent-sdk/src/mcp_format.rs: ResponseShape enum, detect_shape(), McpResponseFormat (Json/Markdown/Xml), ByShapeConfig, ShapeFormatConfig with recommended() preset, format_mcp_result(). Re-exported from hick-agent-sdk root. Used by all four MCP extension crates.
<!-- SECTION:FINAL_SUMMARY:END -->
