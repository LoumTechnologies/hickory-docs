//! Replace transformation for search/replace in content.

use anyhow::Result;
use super::{Transformation, get_attr};

/// Search and replace transformation.
///
/// Replaces all occurrences of a pattern with a replacement string.
///
/// # Attributes
///
/// - `pattern` - The text to search for
/// - `with` - The replacement text
///
/// # Example
///
/// ```xml
/// <hick:replace pattern="foo" with="bar">
///   content with foo here
/// </hick:replace>
/// <!-- Output: "content with bar here" -->
/// ```
pub struct ReplaceTransformation;

impl Transformation for ReplaceTransformation {
    fn transform(&self, input: &str, attrs: &[(String, String)]) -> Result<String> {
        let pattern = get_attr(attrs, "pattern").unwrap_or("");
        let replacement = get_attr(attrs, "with").unwrap_or("");

        if pattern.is_empty() {
            return Ok(input.to_string());
        }

        Ok(input.replace(pattern, replacement))
    }

    fn name(&self) -> &str {
        "replace"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_basic() {
        let t = ReplaceTransformation;
        let attrs = vec![
            ("pattern".to_string(), "foo".to_string()),
            ("with".to_string(), "bar".to_string()),
        ];

        let result = t.transform("foo is foo", &attrs).unwrap();
        assert_eq!(result, "bar is bar");
    }

    #[test]
    fn replace_empty_pattern_returns_input() {
        let t = ReplaceTransformation;
        let attrs = vec![
            ("pattern".to_string(), "".to_string()),
            ("with".to_string(), "bar".to_string()),
        ];

        let result = t.transform("unchanged", &attrs).unwrap();
        assert_eq!(result, "unchanged");
    }

    #[test]
    fn replace_no_match_returns_input() {
        let t = ReplaceTransformation;
        let attrs = vec![
            ("pattern".to_string(), "nomatch".to_string()),
            ("with".to_string(), "bar".to_string()),
        ];

        let result = t.transform("unchanged", &attrs).unwrap();
        assert_eq!(result, "unchanged");
    }

    #[test]
    fn replace_with_empty_string() {
        let t = ReplaceTransformation;
        let attrs = vec![
            ("pattern".to_string(), "remove".to_string()),
            ("with".to_string(), "".to_string()),
        ];

        let result = t.transform("remove this remove", &attrs).unwrap();
        assert_eq!(result, " this ");
    }
}
