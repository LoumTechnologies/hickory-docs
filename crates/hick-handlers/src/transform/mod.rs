//! Content transformation system for hick pipelines.
//!
//! Transformations are XML elements that evaluate their children, apply a
//! transformation function, and emit the result. This enables powerful
//! content manipulation like search/replace, code formatting, etc.
//!
//! # Example
//!
//! ```xml
//! <hick:replace pattern="foo" with="bar">
//!   content with foo becomes content with bar
//! </hick:replace>
//! ```

mod replace;
mod format;

pub use replace::ReplaceTransformation;
pub use format::FormatTransformation;

use anyhow::Result;

// ---------------------------------------------------------------------------
// Transformation Trait
// ---------------------------------------------------------------------------

/// Trait for content transformations.
///
/// A transformation takes string input (evaluated children), applies a
/// transformation function, and returns the transformed string.
pub trait Transformation: Send + Sync {
    /// Apply the transformation to input content.
    ///
    /// # Arguments
    ///
    /// * `input` - The evaluated children content as a string
    /// * `attrs` - Attributes from the transformation tag
    ///
    /// # Returns
    ///
    /// The transformed content string.
    fn transform(&self, input: &str, attrs: &[(String, String)]) -> Result<String>;

    /// The name of this transformation (for debugging/logging).
    fn name(&self) -> &str;
}

// ---------------------------------------------------------------------------
// Helper Functions
// ---------------------------------------------------------------------------

/// Get an attribute value from a list of attributes.
pub fn get_attr<'a>(attrs: &'a [(String, String)], name: &str) -> Option<&'a str> {
    attrs
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.as_str())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    struct UppercaseTransform;

    impl Transformation for UppercaseTransform {
        fn transform(&self, input: &str, _attrs: &[(String, String)]) -> Result<String> {
            Ok(input.to_uppercase())
        }

        fn name(&self) -> &str {
            "uppercase"
        }
    }

    #[test]
    fn transformation_basic() {
        let t = UppercaseTransform;
        let result = t.transform("hello world", &[]).unwrap();
        assert_eq!(result, "HELLO WORLD");
    }

    #[test]
    fn get_attr_finds_value() {
        let attrs = vec![
            ("pattern".to_string(), "foo".to_string()),
            ("with".to_string(), "bar".to_string()),
        ];

        assert_eq!(get_attr(&attrs, "pattern"), Some("foo"));
        assert_eq!(get_attr(&attrs, "with"), Some("bar"));
        assert_eq!(get_attr(&attrs, "missing"), None);
    }
}
