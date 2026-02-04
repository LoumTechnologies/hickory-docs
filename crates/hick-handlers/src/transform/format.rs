//! Format transformation for code formatting.
//!
//! This is a placeholder that provides basic whitespace normalization.
//! In the future, this could integrate with formatters like rustfmt, prettier, etc.

use anyhow::Result;
use super::{Transformation, get_attr};

/// Code formatting transformation.
///
/// Currently provides basic whitespace normalization. Future versions may
/// integrate with language-specific formatters.
///
/// # Attributes
///
/// - `lang` - The language to format (rust, json, etc.)
///
/// # Example
///
/// ```xml
/// <hick:format lang="rust">
///   fn main(){println!("hi")}
/// </hick:format>
/// ```
pub struct FormatTransformation;

impl Transformation for FormatTransformation {
    fn transform(&self, input: &str, attrs: &[(String, String)]) -> Result<String> {
        let _lang = get_attr(attrs, "lang").unwrap_or("text");

        // For now, just trim and normalize line endings
        // Future: integrate with actual formatters based on lang
        let trimmed = input.trim();

        // Normalize line endings to LF
        let normalized = trimmed.replace("\r\n", "\n");

        Ok(normalized)
    }

    fn name(&self) -> &str {
        "format"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_trims_whitespace() {
        let t = FormatTransformation;
        let attrs = vec![("lang".to_string(), "rust".to_string())];

        let result = t.transform("  fn main() {}  ", &attrs).unwrap();
        assert_eq!(result, "fn main() {}");
    }

    #[test]
    fn format_normalizes_line_endings() {
        let t = FormatTransformation;
        let attrs = vec![];

        let result = t.transform("line1\r\nline2\r\n", &attrs).unwrap();
        assert_eq!(result, "line1\nline2");
    }
}
