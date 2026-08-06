//! Semantic equivalence checker for hick pipelines.
//!
//! Verifies that two hick documents produce identical output for all
//! possible parameter/feature combinations, enabling safe refactoring.
//!
//! # Algorithm
//!
//! 1. Extract features and variables from both documents
//! 2. Generate the power set of feature combinations (up to a limit)
//! 3. For each combination, run both pipelines
//! 4. Compare output files (paths and content)
//! 5. Report any differences with diffs
//!
//! # Example
//!
//! ```bash
//! hick-equiv original.hick refactored.hick --max-combinations 1000
//! ```

use std::collections::{BTreeSet, HashMap, HashSet};

use anyhow::Result;
use hick_lang::HickNode;

// ---------------------------------------------------------------------------
// Configuration Extraction
// ---------------------------------------------------------------------------

/// Configuration extracted from a hick document.
#[derive(Debug, Clone, Default)]
pub struct HickConfiguration {
    /// Feature names defined in the document.
    pub features: Vec<String>,
    /// Variable names with optional default values.
    pub variables: Vec<(String, Option<String>)>,
}

/// Extract configuration (features and variables) from a hick source.
pub fn extract_configuration(source: &str) -> Result<HickConfiguration> {
    let doc = hick_lang::parse(source).map_err(|e| anyhow::anyhow!("parse error: {e}"))?;

    let mut config = HickConfiguration::default();
    extract_from_nodes(&doc.nodes, &mut config);

    Ok(config)
}

fn extract_from_nodes(nodes: &[HickNode], config: &mut HickConfiguration) {
    for node in nodes {
        if let HickNode::Tag(tag) = node {
            match tag.name.as_str() {
                "feature" => {
                    if let Some(name) = tag
                        .attributes
                        .iter()
                        .find(|(k, _)| k == "name")
                        .map(|(_, v)| v.clone())
                        && !config.features.contains(&name)
                    {
                        config.features.push(name);
                    }
                }
                "var" => {
                    if let Some(name) = tag
                        .attributes
                        .iter()
                        .find(|(k, _)| k == "name")
                        .map(|(_, v)| v.clone())
                    {
                        // Get default value from text children
                        let default_value: String = tag
                            .children
                            .iter()
                            .filter_map(|c| match c {
                                HickNode::Text(t, _) => Some(t.as_str()),
                                _ => None,
                            })
                            .collect();
                        let default = if default_value.is_empty() {
                            None
                        } else {
                            Some(default_value)
                        };

                        if !config.variables.iter().any(|(n, _)| n == &name) {
                            config.variables.push((name, default));
                        }
                    }
                }
                _ => {}
            }
            // Recurse into children
            extract_from_nodes(&tag.children, config);
        }
    }
}

// ---------------------------------------------------------------------------
// Combination Generation
// ---------------------------------------------------------------------------

/// Parameters for a single pipeline run.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RunParams {
    /// Enabled features (comma-separated string).
    pub features: String,
    /// Variable values.
    pub variables: Vec<(String, String)>,
}

impl RunParams {
    /// Convert to the parameter format expected by run_pipeline.
    pub fn to_params(&self) -> Vec<(String, String)> {
        let mut params = self.variables.clone();
        if !self.features.is_empty() {
            params.push(("features".to_string(), self.features.clone()));
        }
        params
    }
}

/// Generate all feature combinations up to max_combinations.
///
/// Returns the power set of feature names (all subsets), limited by max_combinations.
pub fn generate_combinations(
    config1: &HickConfiguration,
    config2: &HickConfiguration,
    max_combinations: usize,
) -> Vec<RunParams> {
    // Merge features from both configs
    let mut all_features: BTreeSet<String> = BTreeSet::new();
    all_features.extend(config1.features.iter().cloned());
    all_features.extend(config2.features.iter().cloned());

    let features_vec: Vec<String> = all_features.into_iter().collect();
    let n = features_vec.len();

    // Generate power set (all subsets of features)
    let total_combinations = 1usize << n; // 2^n

    let mut combinations = Vec::new();
    for i in 0..total_combinations.min(max_combinations) {
        let mut enabled = Vec::new();
        for (j, feature) in features_vec.iter().enumerate() {
            if (i >> j) & 1 == 1 {
                enabled.push(feature.clone());
            }
        }

        combinations.push(RunParams {
            features: enabled.join(","),
            variables: Vec::new(), // Variables can be added if needed
        });
    }

    combinations
}

// ---------------------------------------------------------------------------
// Output Comparison
// ---------------------------------------------------------------------------

/// Difference between two outputs.
#[derive(Debug, Clone)]
pub struct OutputDiff {
    /// The file path.
    pub path: String,
    /// The kind of difference.
    pub kind: DiffKind,
}

/// Kind of difference found.
#[derive(Debug, Clone)]
pub enum DiffKind {
    /// File exists only in the first output.
    OnlyInFirst,
    /// File exists only in the second output.
    OnlyInSecond,
    /// File content differs.
    ContentDiffers {
        first_content: String,
        second_content: String,
    },
}

/// Compare two sets of output files.
pub fn compare_outputs(
    files1: &HashMap<String, hick_exec::node::FileContent>,
    files2: &HashMap<String, hick_exec::node::FileContent>,
) -> Vec<OutputDiff> {
    let mut diffs = Vec::new();

    let paths1: HashSet<_> = files1.keys().collect();
    let paths2: HashSet<_> = files2.keys().collect();

    // Files only in first
    for path in paths1.difference(&paths2) {
        diffs.push(OutputDiff {
            path: (*path).clone(),
            kind: DiffKind::OnlyInFirst,
        });
    }

    // Files only in second
    for path in paths2.difference(&paths1) {
        diffs.push(OutputDiff {
            path: (*path).clone(),
            kind: DiffKind::OnlyInSecond,
        });
    }

    // Files in both - compare content
    for path in paths1.intersection(&paths2) {
        let content1 = files1.get(*path).unwrap();
        let content2 = files2.get(*path).unwrap();

        let s1 = content1.to_string();
        let s2 = content2.to_string();

        if s1 != s2 {
            diffs.push(OutputDiff {
                path: (*path).clone(),
                kind: DiffKind::ContentDiffers {
                    first_content: s1,
                    second_content: s2,
                },
            });
        }
    }

    diffs
}

// ---------------------------------------------------------------------------
// Equivalence Check Result
// ---------------------------------------------------------------------------

/// Result of an equivalence check.
#[derive(Debug)]
pub struct EquivalenceResult {
    /// Whether the documents are equivalent.
    pub equivalent: bool,
    /// Total combinations tested.
    pub combinations_tested: usize,
    /// Failing combinations (params -> diffs).
    pub failures: Vec<(RunParams, Vec<OutputDiff>)>,
}

/// Run equivalence check between two documents.
pub async fn check_equivalence(
    source1: &str,
    source2: &str,
    max_combinations: usize,
) -> Result<EquivalenceResult> {
    // Extract configurations
    let config1 = extract_configuration(source1)?;
    let config2 = extract_configuration(source2)?;

    // Generate combinations
    let combinations = generate_combinations(&config1, &config2, max_combinations);
    let combinations_tested = combinations.len();

    let mut failures = Vec::new();

    // Test each combination
    for params in combinations {
        let params_vec = params.to_params();

        // Run both pipelines
        let result1 = crate::run_pipeline(&[("source1.hick", source1)], &params_vec).await?;
        let result2 = crate::run_pipeline(&[("source2.hick", source2)], &params_vec).await?;

        // Compare outputs
        let diffs = compare_outputs(&result1.files, &result2.files);

        if !diffs.is_empty() {
            failures.push((params, diffs));
        }
    }

    Ok(EquivalenceResult {
        equivalent: failures.is_empty(),
        combinations_tested,
        failures,
    })
}

// ---------------------------------------------------------------------------
// Diff Formatting
// ---------------------------------------------------------------------------

/// Format a diff for display.
pub fn format_diff(diff: &OutputDiff) -> String {
    match &diff.kind {
        DiffKind::OnlyInFirst => {
            format!("- {} (only in first document)", diff.path)
        }
        DiffKind::OnlyInSecond => {
            format!("+ {} (only in second document)", diff.path)
        }
        DiffKind::ContentDiffers {
            first_content,
            second_content,
        } => {
            let mut output = format!("~ {} (content differs)\n", diff.path);

            // Simple line-by-line diff
            let lines1: Vec<&str> = first_content.lines().collect();
            let lines2: Vec<&str> = second_content.lines().collect();

            let max_lines = lines1.len().max(lines2.len());
            for i in 0..max_lines {
                let l1 = lines1.get(i).copied().unwrap_or("");
                let l2 = lines2.get(i).copied().unwrap_or("");

                if l1 != l2 {
                    if !l1.is_empty() {
                        output.push_str(&format!("  - {}\n", l1));
                    }
                    if !l2.is_empty() {
                        output.push_str(&format!("  + {}\n", l2));
                    }
                }
            }

            output
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn hick_doc(body: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
{body}
</hick:doc>"#
        )
    }

    #[test]
    fn extract_features() {
        let src = hick_doc(
            r#"<hick:feature name="auth" description="Auth" />
<hick:feature name="billing" description="Billing" />"#,
        );

        let config = extract_configuration(&src).unwrap();
        assert_eq!(config.features, vec!["auth", "billing"]);
    }

    #[test]
    fn extract_variables() {
        let src = hick_doc(
            r#"<hick:var name="version">1.0.0</hick:var>
<hick:var name="name">MyApp</hick:var>"#,
        );

        let config = extract_configuration(&src).unwrap();
        assert_eq!(config.variables.len(), 2);
        assert_eq!(
            config.variables[0],
            ("version".to_string(), Some("1.0.0".to_string()))
        );
        assert_eq!(
            config.variables[1],
            ("name".to_string(), Some("MyApp".to_string()))
        );
    }

    #[test]
    fn generate_power_set() {
        let config = HickConfiguration {
            features: vec!["a".to_string(), "b".to_string()],
            variables: vec![],
        };

        let combinations = generate_combinations(&config, &config, 100);

        // 2^2 = 4 combinations: {}, {a}, {b}, {a,b}
        assert_eq!(combinations.len(), 4);

        let feature_sets: HashSet<_> = combinations.iter().map(|p| p.features.clone()).collect();

        assert!(feature_sets.contains(""));
        assert!(feature_sets.contains("a"));
        assert!(feature_sets.contains("b"));
        assert!(feature_sets.contains("a,b"));
    }

    #[test]
    fn compare_identical_outputs() {
        use hick_exec::node::FileContent;
        let mut files1 = HashMap::new();
        files1.insert(
            "a.txt".to_string(),
            FileContent::Text("content".to_string()),
        );

        let mut files2 = HashMap::new();
        files2.insert(
            "a.txt".to_string(),
            FileContent::Text("content".to_string()),
        );

        let diffs = compare_outputs(&files1, &files2);
        assert!(diffs.is_empty());
    }

    #[test]
    fn compare_different_outputs() {
        use hick_exec::node::FileContent;
        let mut files1 = HashMap::new();
        files1.insert(
            "a.txt".to_string(),
            FileContent::Text("content1".to_string()),
        );
        files1.insert(
            "b.txt".to_string(),
            FileContent::Text("only in first".to_string()),
        );

        let mut files2 = HashMap::new();
        files2.insert(
            "a.txt".to_string(),
            FileContent::Text("content2".to_string()),
        );
        files2.insert(
            "c.txt".to_string(),
            FileContent::Text("only in second".to_string()),
        );

        let diffs = compare_outputs(&files1, &files2);
        assert_eq!(diffs.len(), 3);
    }

    #[tokio::test]
    async fn check_equivalent_documents() {
        let src1 = hick_doc(r#"<hick:file path="out.txt">hello</hick:file>"#);
        let src2 = hick_doc(r#"<hick:file path="out.txt">hello</hick:file>"#);

        let result = check_equivalence(&src1, &src2, 10).await.unwrap();
        assert!(result.equivalent);
        assert!(result.failures.is_empty());
    }

    #[tokio::test]
    async fn check_different_documents() {
        let src1 = hick_doc(r#"<hick:file path="out.txt">hello1</hick:file>"#);
        let src2 = hick_doc(r#"<hick:file path="out.txt">hello2</hick:file>"#);

        let result = check_equivalence(&src1, &src2, 10).await.unwrap();
        assert!(!result.equivalent);
        assert!(!result.failures.is_empty());
    }

    #[tokio::test]
    async fn check_feature_based_equivalence() {
        // These two should be equivalent - they produce the same output
        // for all feature combinations
        let src1 = hick_doc(
            r#"<hick:feature name="auth" description="Auth" />
<hick:file path="out.txt">
<hick:when test="auth">auth enabled</hick:when>
<hick:when test="!auth">auth disabled</hick:when>
</hick:file>"#,
        );

        let src2 = hick_doc(
            r#"<hick:feature name="auth" description="Auth" />
<hick:file path="out.txt">
<hick:when test="auth">auth enabled</hick:when>
<hick:when test="!auth">auth disabled</hick:when>
</hick:file>"#,
        );

        let result = check_equivalence(&src1, &src2, 10).await.unwrap();
        assert!(result.equivalent);
        // Should test 2 combinations: {} and {auth}
        assert_eq!(result.combinations_tested, 2);
    }
}
