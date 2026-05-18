//! MCP tool response format selection and serialization.
//!
//! Provides [`detect_shape`] to classify a JSON value's structure, and
//! [`format_mcp_result`] to serialize it in the configured format for its shape.
//! All four MCP extension crates use this shared helper — format logic lives
//! here, not in individual extensions.

use serde::Deserialize;

// ---------------------------------------------------------------------------
// ResponseShape
// ---------------------------------------------------------------------------

/// Structural classification of an MCP tool response value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseShape {
    /// Primitive JSON value, or single-field object whose value is primitive.
    Scalar,
    /// Flat key-value object (all values are primitives, depth ≤ 1).
    Object,
    /// Object with one or more sub-objects or nested arrays (depth > 1).
    Nested,
    /// Top-level array, or object whose sole field is an array.
    List,
}

/// Detect the structural shape of a JSON value.
///
/// Rules (applied in order):
/// - Top-level `Array` → [`ResponseShape::List`]
/// - Top-level primitive (string/number/bool/null) → [`ResponseShape::Scalar`]
/// - Object with single field whose value is an `Array` → [`ResponseShape::List`]
/// - Object with single field whose value is a primitive → [`ResponseShape::Scalar`]
/// - Object with any sub-object or nested array → [`ResponseShape::Nested`]
/// - Otherwise → [`ResponseShape::Object`]
pub fn detect_shape(value: &serde_json::Value) -> ResponseShape {
    match value {
        serde_json::Value::Array(_) => ResponseShape::List,
        serde_json::Value::Object(map) => {
            if map.len() == 1 {
                let v = map.values().next().unwrap();
                match v {
                    serde_json::Value::Array(_) => return ResponseShape::List,
                    serde_json::Value::Object(_) => {}
                    _ => return ResponseShape::Scalar,
                }
            }
            let has_nested = map
                .values()
                .any(|v| matches!(v, serde_json::Value::Object(_) | serde_json::Value::Array(_)));
            if has_nested {
                ResponseShape::Nested
            } else {
                ResponseShape::Object
            }
        }
        _ => ResponseShape::Scalar,
    }
}

// ---------------------------------------------------------------------------
// McpResponseFormat
// ---------------------------------------------------------------------------

/// Output serialization format for MCP tool results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum McpResponseFormat {
    /// Compact JSON — minimal noise for scalars and flat objects.
    #[default]
    Json,
    /// Nested Markdown headers and bullets — readable for complex objects.
    Markdown,
    /// XML with `<items>/<item>` structure — clear boundaries for arrays.
    Xml,
}

// ---------------------------------------------------------------------------
// ShapeFormatConfig
// ---------------------------------------------------------------------------

/// Per-shape format overrides.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ByShapeConfig {
    /// Format for list-shaped results (top-level arrays or single-array objects).
    pub list: Option<McpResponseFormat>,
    /// Format for scalar-shaped results (primitives or single-field primitive objects).
    pub scalar: Option<McpResponseFormat>,
    /// Format for flat-object results (key-value pairs, all primitive values).
    pub object: Option<McpResponseFormat>,
    /// Format for nested-object results (objects containing sub-objects or arrays).
    pub nested: Option<McpResponseFormat>,
}

/// Shape-aware format configuration.
///
/// ```toml
/// [mcp.response_format]
/// default = "json"
///
/// [mcp.response_format.by_shape]
/// list   = "xml"
/// scalar = "json"
/// object = "json"
/// nested = "markdown"
/// ```
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ShapeFormatConfig {
    /// Fallback format when no shape-specific rule matches.
    #[serde(default)]
    pub default: McpResponseFormat,
    /// Optional per-shape overrides.
    #[serde(rename = "by_shape", default)]
    pub by_shape: ByShapeConfig,
}

impl ShapeFormatConfig {
    /// Return the format to use for the given shape.
    pub fn format_for(&self, shape: ResponseShape) -> &McpResponseFormat {
        let override_fmt = match shape {
            ResponseShape::List => self.by_shape.list.as_ref(),
            ResponseShape::Scalar => self.by_shape.scalar.as_ref(),
            ResponseShape::Object => self.by_shape.object.as_ref(),
            ResponseShape::Nested => self.by_shape.nested.as_ref(),
        };
        override_fmt.unwrap_or(&self.default)
    }

    /// Recommended defaults informed by eval results:
    /// list → XML, scalar → JSON, object → JSON, nested → Markdown.
    pub fn recommended() -> Self {
        Self {
            default: McpResponseFormat::Json,
            by_shape: ByShapeConfig {
                list: Some(McpResponseFormat::Xml),
                scalar: Some(McpResponseFormat::Json),
                object: Some(McpResponseFormat::Json),
                nested: Some(McpResponseFormat::Markdown),
            },
        }
    }
}

// ---------------------------------------------------------------------------
// format_mcp_result
// ---------------------------------------------------------------------------

/// Detect the shape of `value` and serialize it in the configured format.
///
/// All four MCP extension crates call this function — format logic is not
/// duplicated per extension.
pub fn format_mcp_result(value: &serde_json::Value, config: &ShapeFormatConfig) -> String {
    let shape = detect_shape(value);
    let format = config.format_for(shape);
    serialize(value, format)
}

fn serialize(value: &serde_json::Value, format: &McpResponseFormat) -> String {
    match format {
        McpResponseFormat::Json => serde_json::to_string(value).unwrap_or_default(),
        McpResponseFormat::Markdown => to_markdown(value, 3),
        McpResponseFormat::Xml => to_xml(value),
    }
}

// ---------------------------------------------------------------------------
// Markdown serializer
// ---------------------------------------------------------------------------

fn to_markdown(value: &serde_json::Value, depth: usize) -> String {
    let heading = "#".repeat(depth);
    match value {
        serde_json::Value::Object(map) => {
            let mut out = String::new();
            for (k, v) in map {
                match v {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        out.push_str(&format!("{heading} {k}\n"));
                        out.push_str(&to_markdown(v, (depth + 1).min(6)));
                        out.push('\n');
                    }
                    _ => {
                        out.push_str(&format!("- **{k}**: {}\n", scalar_to_str(v)));
                    }
                }
            }
            out
        }
        serde_json::Value::Array(arr) => {
            let mut out = String::new();
            for item in arr {
                match item {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        out.push_str(&to_markdown(item, depth));
                        out.push('\n');
                    }
                    _ => {
                        out.push_str(&format!("- {}\n", scalar_to_str(item)));
                    }
                }
            }
            out
        }
        _ => scalar_to_str(value),
    }
}

fn scalar_to_str(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => "null".to_string(),
        _ => v.to_string(),
    }
}

// ---------------------------------------------------------------------------
// XML serializer
// ---------------------------------------------------------------------------

fn to_xml(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Array(arr) => {
            let mut out = String::from("<items>");
            for item in arr {
                out.push_str("<item>");
                out.push_str(&value_to_xml_content(item));
                out.push_str("</item>");
            }
            out.push_str("</items>");
            out
        }
        serde_json::Value::Object(map) if map.len() == 1 => {
            let (key, v) = map.iter().next().unwrap();
            if let serde_json::Value::Array(arr) = v {
                let safe_key = xml_tag_name(key);
                let mut out = format!("<{safe_key}>");
                for item in arr {
                    out.push_str("<item>");
                    out.push_str(&value_to_xml_content(item));
                    out.push_str("</item>");
                }
                out.push_str(&format!("</{safe_key}>"));
                return out;
            }
            value_to_xml_content(value)
        }
        _ => value_to_xml_content(value),
    }
}

fn value_to_xml_content(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => {
            let mut out = String::new();
            for (k, v) in map {
                let tag = xml_tag_name(k);
                out.push_str(&format!("<{tag}>"));
                out.push_str(&value_to_xml_content(v));
                out.push_str(&format!("</{tag}>"));
            }
            out
        }
        serde_json::Value::Array(arr) => {
            let mut out = String::new();
            for item in arr {
                out.push_str("<item>");
                out.push_str(&value_to_xml_content(item));
                out.push_str("</item>");
            }
            out
        }
        _ => xml_escape(&scalar_to_str(value)),
    }
}

fn xml_tag_name(s: &str) -> String {
    s.replace(' ', "_")
        .replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-' && c != '.', "_")
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── detect_shape ──────────────────────────────────────────────────────────

    #[test]
    fn shape_of_primitives_is_scalar() {
        for v in [json!(42), json!("hello"), json!(true), json!(null)] {
            assert_eq!(detect_shape(&v), ResponseShape::Scalar, "value: {v}");
        }
    }

    #[test]
    fn single_field_primitive_object_is_scalar() {
        assert_eq!(detect_shape(&json!({"exit_code": 0})), ResponseShape::Scalar);
        assert_eq!(detect_shape(&json!({"value": "ok"})), ResponseShape::Scalar);
    }

    #[test]
    fn top_level_array_is_list() {
        assert_eq!(detect_shape(&json!([1, 2, 3])), ResponseShape::List);
        assert_eq!(detect_shape(&json!([])), ResponseShape::List);
    }

    #[test]
    fn single_field_array_object_is_list() {
        assert_eq!(
            detect_shape(&json!({"items": [1, 2, 3]})),
            ResponseShape::List
        );
    }

    #[test]
    fn flat_multi_field_object_is_object() {
        assert_eq!(
            detect_shape(&json!({"number": 42, "url": "https://github.com"})),
            ResponseShape::Object
        );
    }

    #[test]
    fn object_with_sub_object_is_nested() {
        assert_eq!(
            detect_shape(&json!({"title": "Fix", "author": {"login": "nate"}})),
            ResponseShape::Nested
        );
    }

    #[test]
    fn object_with_nested_array_is_nested() {
        assert_eq!(
            detect_shape(&json!({"title": "Fix", "labels": ["bug", "p1"]})),
            ResponseShape::Nested
        );
    }

    // ── ShapeFormatConfig ─────────────────────────────────────────────────────

    #[test]
    fn config_default_only_uses_default_for_all_shapes() {
        let config = ShapeFormatConfig::default();
        for shape in [
            ResponseShape::List,
            ResponseShape::Scalar,
            ResponseShape::Object,
            ResponseShape::Nested,
        ] {
            assert_eq!(
                config.format_for(shape),
                &McpResponseFormat::Json,
                "shape: {shape:?}"
            );
        }
    }

    #[test]
    fn recommended_config_routes_correctly() {
        let config = ShapeFormatConfig::recommended();
        assert_eq!(config.format_for(ResponseShape::List), &McpResponseFormat::Xml);
        assert_eq!(config.format_for(ResponseShape::Nested), &McpResponseFormat::Markdown);
        assert_eq!(config.format_for(ResponseShape::Scalar), &McpResponseFormat::Json);
        assert_eq!(config.format_for(ResponseShape::Object), &McpResponseFormat::Json);
    }

    #[test]
    fn toml_deserialization_full() {
        let toml_str = r#"
default = "json"
[by_shape]
list = "xml"
nested = "markdown"
scalar = "json"
"#;
        let config: ShapeFormatConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.default, McpResponseFormat::Json);
        assert_eq!(config.by_shape.list, Some(McpResponseFormat::Xml));
        assert_eq!(config.by_shape.nested, Some(McpResponseFormat::Markdown));
        assert_eq!(config.by_shape.scalar, Some(McpResponseFormat::Json));
        assert_eq!(config.by_shape.object, None);
    }

    #[test]
    fn toml_deserialization_default_only() {
        let config: ShapeFormatConfig = toml::from_str(r#"default = "markdown""#).unwrap();
        assert_eq!(config.default, McpResponseFormat::Markdown);
        assert!(config.by_shape.list.is_none());
    }

    // ── format_mcp_result ─────────────────────────────────────────────────────

    #[test]
    fn scalar_formatted_as_json() {
        let v = json!({"exit_code": 0});
        let result = format_mcp_result(&v, &ShapeFormatConfig::default());
        assert_eq!(result, r#"{"exit_code":0}"#);
    }

    #[test]
    fn list_formatted_as_xml_via_recommended() {
        let v = json!([{"number": 42, "url": "https://github.com/"}]);
        let result = format_mcp_result(&v, &ShapeFormatConfig::recommended());
        assert!(result.starts_with("<items>"), "got: {result}");
        assert!(result.contains("<number>42</number>"));
        assert!(result.contains("<url>"));
    }

    #[test]
    fn nested_formatted_as_markdown_via_recommended() {
        let v = json!({"title": "Fix bug", "author": {"login": "nate"}});
        let result = format_mcp_result(&v, &ShapeFormatConfig::recommended());
        assert!(result.contains("**title**"), "got: {result}");
        assert!(result.contains("### author") || result.contains("#### author"));
        assert!(result.contains("**login**"));
    }

    #[test]
    fn xml_escapes_special_chars() {
        let v = json!({"msg": "<hello> & 'world'"});
        let result = serialize(&v, &McpResponseFormat::Xml);
        assert!(result.contains("&lt;hello&gt;"));
        assert!(result.contains("&amp;"));
    }

    #[test]
    fn single_array_field_object_xml_uses_field_name_as_root() {
        let v = json!({"tools": [{"name": "read_file"}, {"name": "write_file"}]});
        let result = serialize(&v, &McpResponseFormat::Xml);
        assert!(result.starts_with("<tools>"), "got: {result}");
        assert!(result.contains("<item>"));
    }

    #[test]
    fn object_formatted_as_json_via_recommended() {
        let v = json!({"number": 42, "url": "https://github.com"});
        let result = format_mcp_result(&v, &ShapeFormatConfig::recommended());
        // Object shape → JSON by recommended config
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["number"], 42);
    }
}
