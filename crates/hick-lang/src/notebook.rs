//! Rich stdout records survive the ordinary cell recording and weave paths.
use serde_json::Value;
pub const DISPLAY_PREFIX: &str = "\u{1e}HICKORY_DISPLAY ";

pub fn render_output(output: &str) -> String {
    output
        .split('\n')
        .map(|line| {
            line.strip_prefix(DISPLAY_PREFIX)
                .and_then(|json| serde_json::from_str::<Value>(json).ok())
                .and_then(|v| render_value(&v))
                .unwrap_or_else(|| line.to_string())
        })
        .collect::<Vec<_>>()
        .join("\n")
}
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('\n', "<br>")
        .replace('\r', "")
}
fn render_value(v: &Value) -> Option<String> {
    match v.get("kind")?.as_str()? {
        "table" => {
            let schema = v.get("schema")?.as_array()?;
            let rows = v.get("rows")?.as_array()?;
            let mut out = String::from("\n");
            out.push('|');
            for col in schema {
                out.push_str(&format!(
                    " {} ({}) |",
                    escape(col.get("name")?.as_str()?),
                    escape(col.get("dtype")?.as_str()?)
                ));
            }
            out.push_str(&format!("\n|{}\n", " --- |".repeat(schema.len())));
            for row in rows {
                out.push('|');
                for value in row.as_array()? {
                    let s = value
                        .as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| value.to_string());
                    out.push_str(&format!(" {} |", escape(&s)));
                }
                out.push('\n');
            }
            if v.get("has_more").and_then(Value::as_bool) == Some(true) {
                out.push_str("\nPreview truncated.\n");
            }
            Some(out)
        }
        "image" => {
            let mime = v.get("mime")?.as_str()?;
            let data = v.get("data")?.as_str()?;
            if !matches!(
                mime,
                "image/png" | "image/jpeg" | "image/webp" | "image/svg+xml"
            ) || !data
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"+/=".contains(&c))
            {
                return None;
            }
            Some(format!("\n![Cell output](data:{mime};base64,{data})\n"))
        }
        "text" => Some(escape(v.get("text")?.as_str()?)),
        _ => None,
    }
}

/// Stable cache identity for a Python cell, including its display runtime.
pub fn python_command(source: &str) -> String {
    let program = format!(
        "{}\n# HICKORY_CELL_SOURCE\n{}",
        include_str!("../../hick-exec/python/notebook.py"),
        source
    );
    let hex: String = program
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    format!("python3 -c \"exec(bytes.fromhex('{hex}'))\"")
}

/// The internal command embeds Python as hex to prevent shell interpolation.
/// Show the author's source when weaving a command transcript.
pub fn python_program(command: &str) -> Option<String> {
    let hex = command
        .strip_prefix("python3 -c \"exec(bytes.fromhex('")?
        .strip_suffix("'))\"")?;
    if !hex.is_ascii() || !hex.len().is_multiple_of(2) {
        return None;
    }
    let bytes: Option<Vec<u8>> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect();
    String::from_utf8(bytes?).ok()
}

pub fn source_command(command: &str) -> Option<String> {
    Some(
        python_program(command)?
            .split_once("\n# HICKORY_CELL_SOURCE\n")?
            .1
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn python_source_and_runtime_survive_the_command_boundary() {
        let source = "display(\"hello\")\nprint('$HOME; `no shell`')\n";
        let command = python_command(source);
        assert_eq!(source_command(&command).as_deref(), Some(source));
        assert!(python_program(&command).unwrap().contains("def display("));
        assert!(python_program("python3 -c \"exec(bytes.fromhex('é'))\"").is_none());
    }
    #[test]
    fn rich_table_weaves_schema_null_and_escaped_cells() {
        let output = format!(
            "before\n{DISPLAY_PREFIX}{}\nafter",
            r#"{"kind":"table","schema":[{"name":"x","dtype":"String"}],"rows":[[null],["<script>|hi"]],"has_more":true}"#
        );
        let result = render_output(&output);
        assert!(result.contains("x (String)"));
        assert!(result.contains(" null |"));
        assert!(result.contains("&lt;script&gt;\\|hi"));
        assert!(result.contains("Preview truncated"));
        assert!(!result.contains(DISPLAY_PREFIX));
    }
    #[test]
    fn malformed_records_remain_visible_and_unsafe_images_do_not_render() {
        let raw = format!("{DISPLAY_PREFIX}not json");
        assert_eq!(render_output(&raw), raw);
        assert_eq!(
            render_value(&serde_json::json!({"kind":"image","mime":"text/html","data":"abcd"})),
            None
        );
    }
}
