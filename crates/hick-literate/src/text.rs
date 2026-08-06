//! Text substitution and interpolation.
//!
//! Pure(ish) text transformation functions used by both the pipeline core
//! and the weave renderer: variable interpolation, substitution application,
//! exclusion filtering.

use std::collections::HashMap;

use hick_case::{apply_case_variant, generate_case_variants};
use hick_exec::node::FileContent;
use hick_exec::state::MultiDocumentState;
use log::{debug, warn};

/// Interpolate variables in a file path using `{{variable}}` or `{{variable:variant}}` syntax.
///
/// Examples:
/// - `Sources/{{project_name}}/App.swift` with project_name="MyApp" -> `Sources/MyApp/App.swift`
/// - `{{project_name:snake_case}}.rs` with project_name="MyApp" -> `my_app.rs`
pub(crate) fn interpolate_path(path: &str, state: &MultiDocumentState) -> String {
    let mut result = path.to_string();
    let mut start = 0;

    while let Some(open_pos) = result[start..].find("{{") {
        let open_pos = start + open_pos;
        if let Some(close_pos) = result[open_pos..].find("}}") {
            let close_pos = open_pos + close_pos;
            let placeholder = &result[open_pos + 2..close_pos];

            // Check for variant syntax: variable:variant
            let (var_name, variant) = if let Some(colon_pos) = placeholder.find(':') {
                (
                    placeholder[..colon_pos].trim(),
                    Some(placeholder[colon_pos + 1..].trim()),
                )
            } else {
                (placeholder.trim(), None)
            };

            // Resolve the variable
            let resolved = if let Some(value) = state.resolve_var(var_name) {
                if let Some(variant_name) = variant {
                    // Apply case transformation
                    apply_case_variant(&value, variant_name).unwrap_or(value)
                } else {
                    value
                }
            } else {
                // Variable not found, leave placeholder as-is
                warn!("Path variable '{var_name}' not found");
                start = close_pos + 2;
                continue;
            };

            // Replace the placeholder with the resolved value
            result = format!(
                "{}{}{}",
                &result[..open_pos],
                resolved,
                &result[close_pos + 2..]
            );
            start = open_pos + resolved.len();
        } else {
            // No closing }}, move past this {{
            start = open_pos + 2;
        }
    }

    // Also apply regular substitutions to the path
    apply_substitutions(&result, state)
}

/// Filter out files that match any exclusion pattern.
///
/// Patterns use glob syntax:
/// - `*` matches any sequence of characters (except `/`)
/// - `**` matches any sequence of characters (including `/`)
/// - `?` matches any single character
///
/// Examples:
/// - `*.template-only` — exclude files ending with `.template-only`
/// - `docs/internal/**` — exclude everything under `docs/internal/`
/// - `TEMPLATE_README.md` — exclude a specific file
pub(crate) fn apply_exclusions(
    files: HashMap<String, FileContent>,
    patterns: &[String],
) -> HashMap<String, FileContent> {
    if patterns.is_empty() {
        return files;
    }

    // Compile glob patterns
    let compiled_patterns: Vec<glob::Pattern> = patterns
        .iter()
        .filter_map(|p| match glob::Pattern::new(p) {
            Ok(pattern) => Some(pattern),
            Err(e) => {
                warn!("Invalid exclusion pattern '{}': {}", p, e);
                None
            }
        })
        .collect();

    if compiled_patterns.is_empty() {
        return files;
    }

    // Filter out files matching any exclusion pattern
    files
        .into_iter()
        .filter(|(path, _)| {
            let excluded = compiled_patterns
                .iter()
                .any(|pattern| pattern.matches(path));
            if excluded {
                debug!("Excluding file: {}", path);
            }
            !excluded
        })
        .collect()
}

/// Resolve {{variable}} and {{variable:variant}} references in a string.
/// Returns the string with all variables replaced by their values.
fn resolve_value_variables(text: &str, state: &MultiDocumentState) -> String {
    let mut result = text.to_string();
    let mut start = 0;

    while let Some(open_pos) = result[start..].find("{{") {
        let open_pos = start + open_pos;
        if let Some(close_pos) = result[open_pos..].find("}}") {
            let close_pos = open_pos + close_pos;
            let placeholder = &result[open_pos + 2..close_pos];

            // Check for variant syntax: variable:variant
            let (var_name, variant) = if let Some(colon_pos) = placeholder.find(':') {
                (
                    placeholder[..colon_pos].trim(),
                    Some(placeholder[colon_pos + 1..].trim()),
                )
            } else {
                (placeholder.trim(), None)
            };

            // Resolve the variable
            if let Some(value) = state.resolve_var(var_name) {
                let resolved = if let Some(variant_name) = variant {
                    apply_case_variant(&value, variant_name).unwrap_or(value)
                } else {
                    value
                };
                result = format!(
                    "{}{}{}",
                    &result[..open_pos],
                    resolved,
                    &result[close_pos + 2..]
                );
                start = open_pos + resolved.len();
            } else {
                // Variable not found, leave as-is
                start = close_pos + 2;
            }
        } else {
            start = open_pos + 2;
        }
    }

    result
}

/// A segment of text after substitution, tracking whether it was transformed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TransformSegment {
    /// The text content of this segment.
    pub text: String,
    /// Where this segment came from.
    pub origin: SegmentOrigin,
}

/// Origin of a transform segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SegmentOrigin {
    /// Text that passed through unchanged.
    Passthrough,
    /// Text that was produced by a substitution.
    Substituted { pattern: String, value: String },
}

/// Apply substitutions and return segments tracking which parts changed.
///
/// Each segment records whether it was a passthrough (original text) or
/// was produced by a substitution (with the pattern and value that produced it).
pub(crate) fn apply_substitutions_segmented(
    text: &str,
    state: &MultiDocumentState,
) -> Vec<TransformSegment> {
    let substitutions = state.get_substitutions();
    if substitutions.is_empty() {
        return vec![TransformSegment {
            text: text.to_string(),
            origin: SegmentOrigin::Passthrough,
        }];
    }

    // Collect all pattern -> value pairs, expanding variants
    let mut replacements: Vec<(String, String)> = Vec::new();
    for sub in &substitutions {
        let resolved_value = resolve_value_variables(&sub.value, state);
        if sub.variants {
            let variants = generate_case_variants(&sub.pattern, &resolved_value);
            replacements.extend(variants);
        } else {
            replacements.push((sub.pattern.clone(), resolved_value));
        }
    }

    // Sort by pattern length descending (longest first)
    replacements.sort_by_key(|r| std::cmp::Reverse(r.0.len()));

    // Build segments by scanning for pattern occurrences
    segment_text(text, &replacements)
}

/// Scan text for pattern matches and split into segments.
fn segment_text(text: &str, replacements: &[(String, String)]) -> Vec<TransformSegment> {
    if replacements.is_empty() || text.is_empty() {
        return vec![TransformSegment {
            text: text.to_string(),
            origin: SegmentOrigin::Passthrough,
        }];
    }

    let mut segments = Vec::new();
    let mut pos = 0;

    while pos < text.len() {
        // Find the earliest and longest pattern match starting at or after pos
        let mut best_match: Option<(usize, &str, &str)> = None; // (start, pattern, value)

        for (pattern, value) in replacements {
            if pattern.is_empty() {
                continue;
            }
            if let Some(found_pos) = text[pos..].find(pattern.as_str()) {
                let abs_pos = pos + found_pos;
                match best_match {
                    None => best_match = Some((abs_pos, pattern, value)),
                    Some((best_pos, best_pat, _)) => {
                        // Prefer earlier match, then longer pattern
                        if abs_pos < best_pos
                            || (abs_pos == best_pos && pattern.len() > best_pat.len())
                        {
                            best_match = Some((abs_pos, pattern, value));
                        }
                    }
                }
            }
        }

        match best_match {
            Some((match_start, pattern, value)) => {
                // Emit passthrough segment before the match
                if match_start > pos {
                    segments.push(TransformSegment {
                        text: text[pos..match_start].to_string(),
                        origin: SegmentOrigin::Passthrough,
                    });
                }
                // Emit substituted segment
                segments.push(TransformSegment {
                    text: value.to_string(),
                    origin: SegmentOrigin::Substituted {
                        pattern: pattern.to_string(),
                        value: value.to_string(),
                    },
                });
                pos = match_start + pattern.len();
            }
            None => {
                // No more matches — emit the rest as passthrough
                segments.push(TransformSegment {
                    text: text[pos..].to_string(),
                    origin: SegmentOrigin::Passthrough,
                });
                break;
            }
        }
    }

    segments
}

/// Apply all registered substitutions to text content.
///
/// When a substitution has `variants=true`, it generates all case variants
/// (PascalCase, snake_case, etc.) of both the pattern and value, and applies
/// them. Patterns are applied longest-first to avoid partial replacements.
pub(crate) fn apply_substitutions(text: &str, state: &MultiDocumentState) -> String {
    let substitutions = state.get_substitutions();
    if substitutions.is_empty() {
        return text.to_string();
    }

    // Collect all pattern -> value pairs, expanding variants
    let mut replacements: Vec<(String, String)> = Vec::new();
    for sub in &substitutions {
        // Resolve any {{variable}} references in the substitution value
        let resolved_value = resolve_value_variables(&sub.value, state);

        if sub.variants {
            // Generate all case variants
            let variants = generate_case_variants(&sub.pattern, &resolved_value);
            replacements.extend(variants);
        } else {
            // Just the single pattern -> value
            replacements.push((sub.pattern.clone(), resolved_value));
        }
    }

    // Sort by pattern length descending (longest first to avoid partial matches)
    replacements.sort_by_key(|r| std::cmp::Reverse(r.0.len()));

    // Apply all replacements
    let mut result = text.to_string();
    for (pattern, value) in &replacements {
        if !pattern.is_empty() {
            result = result.replace(pattern, value);
        }
    }

    result
}
