//! Case transformation utilities for text substitution patterns.
//!
//! This crate provides functions for converting between common case conventions:
//! - PascalCase (FavoriteApp)
//! - camelCase (favoriteApp)
//! - snake_case (favorite_app)
//! - SCREAMING_SNAKE_CASE (FAVORITE_APP)
//! - kebab-case (favorite-app)
//! - Title Case (Favorite App)
//! - lowercase (favoriteapp)
//! - UPPERCASE (FAVORITEAPP)
//!
//! # Example
//!
//! ```
//! use hick_case::{to_snake_case, to_pascal_case, generate_case_variants};
//!
//! assert_eq!(to_snake_case("FavoriteApp"), "favorite_app");
//! assert_eq!(to_pascal_case("favorite_app"), "FavoriteApp");
//!
//! // Generate all case variants for substitution
//! let variants = generate_case_variants("FavoriteApp", "MyProject");
//! assert!(variants.contains(&("favorite_app".to_string(), "my_project".to_string())));
//! ```

/// Detect word boundaries in a string and return a list of lowercase words.
///
/// Handles:
/// - PascalCase/camelCase boundaries (uppercase letter starts new word)
/// - snake_case/kebab-case separators (_ and -)
/// - Spaces
/// - Consecutive uppercase (e.g., "HTTPServer" -> ["http", "server"])
pub fn detect_words(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current_word = String::new();
    let mut prev_was_upper = false;
    let mut prev_was_separator = true; // start as if after separator

    for ch in s.chars() {
        if ch == '_' || ch == '-' || ch == ' ' {
            // Separator: finish current word
            if !current_word.is_empty() {
                words.push(current_word.to_lowercase());
                current_word.clear();
            }
            prev_was_separator = true;
            prev_was_upper = false;
        } else if ch.is_uppercase() {
            if !prev_was_upper && !prev_was_separator && !current_word.is_empty() {
                // Transition from lowercase to uppercase: new word
                words.push(current_word.to_lowercase());
                current_word.clear();
            } else if prev_was_upper && current_word.len() > 1 {
                // Check if this is the start of a new word after acronym
                // e.g., "HTTPServer" - when we hit 'S', we should split "HTTP" and start "Server"
                // We look ahead by checking if next char would be lowercase
                // For now, we handle this by keeping the acronym together
            }
            current_word.push(ch);
            prev_was_upper = true;
            prev_was_separator = false;
        } else if ch.is_lowercase() {
            if prev_was_upper && current_word.len() > 1 {
                // Transition from uppercase sequence to lowercase
                // e.g., "HTTPServer" -> when we see 'e' after "HTTPS", split to "HTTP" + "Server"
                let last_char = current_word.pop().unwrap();
                if !current_word.is_empty() {
                    words.push(current_word.to_lowercase());
                    current_word.clear();
                }
                current_word.push(last_char);
            }
            current_word.push(ch);
            prev_was_upper = false;
            prev_was_separator = false;
        } else if ch.is_numeric() {
            current_word.push(ch);
            prev_was_upper = false;
            prev_was_separator = false;
        }
        // Ignore other characters
    }

    if !current_word.is_empty() {
        words.push(current_word.to_lowercase());
    }

    words
}

/// Convert to PascalCase (e.g., "favorite_app" -> "FavoriteApp")
pub fn to_pascal_case(s: &str) -> String {
    detect_words(s)
        .iter()
        .map(|w| capitalize_first(w))
        .collect()
}

/// Convert to camelCase (e.g., "favorite_app" -> "favoriteApp")
pub fn to_camel_case(s: &str) -> String {
    let words = detect_words(s);
    if words.is_empty() {
        return String::new();
    }
    let mut result = words[0].clone();
    for word in &words[1..] {
        result.push_str(&capitalize_first(word));
    }
    result
}

/// Convert to snake_case (e.g., "FavoriteApp" -> "favorite_app")
pub fn to_snake_case(s: &str) -> String {
    detect_words(s).join("_")
}

/// Convert to SCREAMING_SNAKE_CASE (e.g., "FavoriteApp" -> "FAVORITE_APP")
pub fn to_screaming_snake_case(s: &str) -> String {
    detect_words(s)
        .iter()
        .map(|w| w.to_uppercase())
        .collect::<Vec<_>>()
        .join("_")
}

/// Convert to kebab-case (e.g., "FavoriteApp" -> "favorite-app")
pub fn to_kebab_case(s: &str) -> String {
    detect_words(s).join("-")
}

/// Convert to Title Case (e.g., "favorite_app" -> "Favorite App")
pub fn to_title_case(s: &str) -> String {
    detect_words(s)
        .iter()
        .map(|w| capitalize_first(w))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convert to lowercase with no separators (e.g., "FavoriteApp" -> "favoriteapp")
pub fn to_lowercase(s: &str) -> String {
    detect_words(s).concat()
}

/// Convert to UPPERCASE with no separators (e.g., "FavoriteApp" -> "FAVORITEAPP")
pub fn to_uppercase(s: &str) -> String {
    detect_words(s).concat().to_uppercase()
}

/// Convert dots to slashes for filesystem paths (e.g., "com.example.app" -> "com/example/app")
///
/// This is useful for converting Java/Kotlin package names to directory paths.
pub fn to_path(s: &str) -> String {
    s.replace('.', "/")
}

/// Apply a named case transformation.
///
/// Supported variants:
/// - "pascal", "pascalcase", "pascal_case" -> PascalCase
/// - "camel", "camelcase", "camel_case" -> camelCase
/// - "snake", "snakecase", "snake_case" -> snake_case
/// - "screaming", "screaming_snake", "screaming_snake_case" -> SCREAMING_SNAKE_CASE
/// - "kebab", "kebabcase", "kebab-case" -> kebab-case
/// - "title", "titlecase", "title_case" -> Title Case
/// - "lower", "lowercase" -> lowercase
/// - "upper", "uppercase" -> UPPERCASE
/// - "path" -> convert dots to slashes (e.g., "com.example.app" -> "com/example/app")
pub fn apply_case_variant(s: &str, variant: &str) -> Option<String> {
    let variant_lower = variant.to_lowercase().replace(['-', '_'], "");
    match variant_lower.as_str() {
        "pascal" | "pascalcase" => Some(to_pascal_case(s)),
        "camel" | "camelcase" => Some(to_camel_case(s)),
        "snake" | "snakecase" => Some(to_snake_case(s)),
        "screaming" | "screamingsnake" | "screamingsnakecase" => Some(to_screaming_snake_case(s)),
        "kebab" | "kebabcase" => Some(to_kebab_case(s)),
        "title" | "titlecase" => Some(to_title_case(s)),
        "lower" | "lowercase" => Some(to_lowercase(s)),
        "upper" | "uppercase" => Some(to_uppercase(s)),
        "path" => Some(to_path(s)),
        _ => None,
    }
}

/// Generate all case variants of a value, paired with the same transformation
/// applied to a pattern. Returns (transformed_pattern, transformed_value) pairs.
///
/// This is used for substitution with variants=true: we detect the case of the
/// pattern and generate corresponding transformations of the value.
///
/// The returned vector is sorted by pattern length (longest first) to avoid
/// partial matches during substitution.
pub fn generate_case_variants(pattern: &str, value: &str) -> Vec<(String, String)> {
    let mut variants = Vec::new();

    // Always include the original pattern -> value mapping
    variants.push((pattern.to_string(), value.to_string()));

    // Generate all standard case variants
    let transformations: &[fn(&str) -> String] = &[
        to_pascal_case,
        to_camel_case,
        to_snake_case,
        to_screaming_snake_case,
        to_kebab_case,
        to_title_case,
        to_lowercase,
        to_uppercase,
    ];

    for transform in transformations {
        let transformed_pattern = transform(pattern);
        let transformed_value = transform(value);

        // Only add if different from original and not already present
        if transformed_pattern != pattern
            && !variants.iter().any(|(p, _)| p == &transformed_pattern)
        {
            variants.push((transformed_pattern, transformed_value));
        }
    }

    // Sort by pattern length descending (longer patterns first to avoid partial matches)
    variants.sort_by_key(|v| std::cmp::Reverse(v.0.len()));

    variants
}

/// Capitalize the first letter of a string.
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Word detection tests
    // -------------------------------------------------------------------------

    #[test]
    fn detect_words_pascal_case() {
        assert_eq!(detect_words("FavoriteApp"), vec!["favorite", "app"]);
    }

    #[test]
    fn detect_words_camel_case() {
        assert_eq!(detect_words("favoriteApp"), vec!["favorite", "app"]);
    }

    #[test]
    fn detect_words_snake_case() {
        assert_eq!(detect_words("favorite_app"), vec!["favorite", "app"]);
    }

    #[test]
    fn detect_words_kebab_case() {
        assert_eq!(detect_words("favorite-app"), vec!["favorite", "app"]);
    }

    #[test]
    fn detect_words_with_spaces() {
        assert_eq!(detect_words("Favorite App"), vec!["favorite", "app"]);
    }

    #[test]
    fn detect_words_acronym() {
        assert_eq!(detect_words("HTTPServer"), vec!["http", "server"]);
    }

    #[test]
    fn detect_words_acronym_at_end() {
        assert_eq!(detect_words("myHTTP"), vec!["my", "http"]);
    }

    #[test]
    fn detect_words_with_numbers() {
        assert_eq!(detect_words("Project2024"), vec!["project2024"]);
    }

    #[test]
    fn detect_words_mixed() {
        assert_eq!(
            detect_words("myFavoriteApp_v2"),
            vec!["my", "favorite", "app", "v2"]
        );
    }

    // -------------------------------------------------------------------------
    // Case transformation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("favorite_app"), "FavoriteApp");
        assert_eq!(to_pascal_case("favoriteApp"), "FavoriteApp");
        assert_eq!(to_pascal_case("FavoriteApp"), "FavoriteApp");
        assert_eq!(to_pascal_case("favorite-app"), "FavoriteApp");
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("favorite_app"), "favoriteApp");
        assert_eq!(to_camel_case("FavoriteApp"), "favoriteApp");
        assert_eq!(to_camel_case("favoriteApp"), "favoriteApp");
        assert_eq!(to_camel_case("FAVORITE_APP"), "favoriteApp");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("FavoriteApp"), "favorite_app");
        assert_eq!(to_snake_case("favoriteApp"), "favorite_app");
        assert_eq!(to_snake_case("favorite-app"), "favorite_app");
        assert_eq!(to_snake_case("Favorite App"), "favorite_app");
    }

    #[test]
    fn test_to_screaming_snake_case() {
        assert_eq!(to_screaming_snake_case("FavoriteApp"), "FAVORITE_APP");
        assert_eq!(to_screaming_snake_case("favorite_app"), "FAVORITE_APP");
        assert_eq!(to_screaming_snake_case("favoriteApp"), "FAVORITE_APP");
    }

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case("FavoriteApp"), "favorite-app");
        assert_eq!(to_kebab_case("favorite_app"), "favorite-app");
        assert_eq!(to_kebab_case("favoriteApp"), "favorite-app");
    }

    #[test]
    fn test_to_title_case() {
        assert_eq!(to_title_case("favorite_app"), "Favorite App");
        assert_eq!(to_title_case("FavoriteApp"), "Favorite App");
        assert_eq!(to_title_case("favoriteApp"), "Favorite App");
    }

    #[test]
    fn test_to_lowercase() {
        assert_eq!(to_lowercase("FavoriteApp"), "favoriteapp");
        assert_eq!(to_lowercase("favorite_app"), "favoriteapp");
    }

    #[test]
    fn test_to_uppercase() {
        assert_eq!(to_uppercase("FavoriteApp"), "FAVORITEAPP");
        assert_eq!(to_uppercase("favorite_app"), "FAVORITEAPP");
    }

    // -------------------------------------------------------------------------
    // apply_case_variant tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_apply_case_variant() {
        assert_eq!(
            apply_case_variant("FavoriteApp", "snake_case"),
            Some("favorite_app".to_string())
        );
        assert_eq!(
            apply_case_variant("favorite_app", "pascal"),
            Some("FavoriteApp".to_string())
        );
        assert_eq!(
            apply_case_variant("com.example.app", "path"),
            Some("com/example/app".to_string())
        );
        assert_eq!(apply_case_variant("test", "unknown_variant"), None);
    }

    #[test]
    fn test_to_path() {
        assert_eq!(to_path("com.example.app"), "com/example/app");
        assert_eq!(to_path("com.example"), "com/example");
        assert_eq!(to_path("app"), "app");
    }

    // -------------------------------------------------------------------------
    // generate_case_variants tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_generate_case_variants() {
        let variants = generate_case_variants("FavoriteApp", "MyProject");

        // Should include the original
        assert!(variants.contains(&("FavoriteApp".to_string(), "MyProject".to_string())));

        // Should include snake_case
        assert!(variants.contains(&("favorite_app".to_string(), "my_project".to_string())));

        // Should include camelCase
        assert!(variants.contains(&("favoriteApp".to_string(), "myProject".to_string())));

        // Should include SCREAMING_SNAKE_CASE
        assert!(variants.contains(&("FAVORITE_APP".to_string(), "MY_PROJECT".to_string())));

        // Should include kebab-case
        assert!(variants.contains(&("favorite-app".to_string(), "my-project".to_string())));

        // Should be sorted by pattern length (longest first)
        let first_pattern_len = variants[0].0.len();
        for (pattern, _) in &variants[1..] {
            assert!(pattern.len() <= first_pattern_len);
        }
    }

    #[test]
    fn test_generate_case_variants_no_duplicates() {
        let variants = generate_case_variants("test", "value");

        // Count unique patterns
        let patterns: Vec<_> = variants.iter().map(|(p, _)| p).collect();
        let unique_patterns: std::collections::HashSet<_> = patterns.iter().collect();
        assert_eq!(patterns.len(), unique_patterns.len());
    }
}
