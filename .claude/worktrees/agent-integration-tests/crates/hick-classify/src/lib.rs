//! Data classification labels and masking specifications.
//!
//! Provides JSON-based classification labels for data columns and rows,
//! along with masking operations for data protection.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[cfg(feature = "host-mask")]
pub mod host_mask;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ClassifyError {
    #[error("invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("access denied: {0}")]
    AccessDenied(String),
}

// ---------------------------------------------------------------------------
// Classification
// ---------------------------------------------------------------------------

/// A set of classification labels represented as a list of arbitrary JSON values.
///
/// Labels are intentionally free-form (not fixed enums) so that policies
/// can evolve without changing the classification schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Classification(Vec<serde_json::Value>);

impl Classification {
    pub fn new(labels: Vec<serde_json::Value>) -> Self {
        Self(labels)
    }

    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn labels(&self) -> &[serde_json::Value] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Merge two classifications, producing the union of their labels.
    /// Duplicate labels (by JSON equality) are deduplicated.
    pub fn union(&self, other: &Classification) -> Classification {
        let mut merged = self.0.clone();
        for label in &other.0 {
            if !merged.contains(label) {
                merged.push(label.clone());
            }
        }
        Classification(merged)
    }

    pub fn to_json_string(&self) -> Result<String, ClassifyError> {
        serde_json::to_string(&self.0).map_err(ClassifyError::InvalidJson)
    }

    pub fn from_json_string(s: &str) -> Result<Self, ClassifyError> {
        let labels: Vec<serde_json::Value> =
            serde_json::from_str(s).map_err(ClassifyError::InvalidJson)?;
        Ok(Self(labels))
    }
}

impl Default for Classification {
    fn default() -> Self {
        Self::empty()
    }
}

// ---------------------------------------------------------------------------
// ColumnClassifications
// ---------------------------------------------------------------------------

/// Per-column classification labels.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnClassifications {
    columns: HashMap<String, Classification>,
}

impl ColumnClassifications {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, column: &str, classification: Classification) {
        self.columns.insert(column.to_string(), classification);
    }

    pub fn get(&self, column: &str) -> Option<&Classification> {
        self.columns.get(column)
    }

    /// Project to a subset of columns.
    pub fn project(&self, columns: &[String]) -> Self {
        let mut projected = HashMap::new();
        for col in columns {
            if let Some(cls) = self.columns.get(col) {
                projected.insert(col.clone(), cls.clone());
            }
        }
        Self { columns: projected }
    }

    /// Merge all column labels into a single classification.
    pub fn union_all(&self) -> Classification {
        let mut result = Classification::empty();
        for cls in self.columns.values() {
            result = result.union(cls);
        }
        result
    }

    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Classification)> {
        self.columns.iter()
    }
}

// ---------------------------------------------------------------------------
// Masking
// ---------------------------------------------------------------------------

/// The type of masking operation to apply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaskOperation {
    /// Replace value with a fixed mask string.
    Mask,
    /// Replace value with null.
    Redact,
    /// Replace value with a BLAKE3 hash.
    Hash,
    /// Bucket numeric values into ranges.
    Bucket,
}

/// Specification for masking a single column.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskSpec {
    /// Column to mask.
    pub column: String,
    /// Masking operation.
    pub operation: MaskOperation,
    /// Classification to assign after masking (replaces original).
    pub target_classification: Option<Classification>,
    /// For Mask operation: the replacement string (default: "***").
    pub mask_value: Option<String>,
    /// For Bucket operation: the bucket size.
    pub bucket_size: Option<f64>,
    /// For Hash operation: optional salt prefix.
    pub hash_salt: Option<String>,
}

impl MaskSpec {
    pub fn mask(column: &str) -> Self {
        Self {
            column: column.to_string(),
            operation: MaskOperation::Mask,
            target_classification: None,
            mask_value: None,
            bucket_size: None,
            hash_salt: None,
        }
    }

    pub fn redact(column: &str) -> Self {
        Self {
            column: column.to_string(),
            operation: MaskOperation::Redact,
            target_classification: None,
            mask_value: None,
            bucket_size: None,
            hash_salt: None,
        }
    }

    pub fn hash(column: &str) -> Self {
        Self {
            column: column.to_string(),
            operation: MaskOperation::Hash,
            target_classification: None,
            mask_value: None,
            bucket_size: None,
            hash_salt: None,
        }
    }

    pub fn bucket(column: &str, size: f64) -> Self {
        Self {
            column: column.to_string(),
            operation: MaskOperation::Bucket,
            target_classification: None,
            mask_value: None,
            bucket_size: Some(size),
            hash_salt: None,
        }
    }

    pub fn with_target_classification(mut self, cls: Classification) -> Self {
        self.target_classification = Some(cls);
        self
    }

    pub fn with_mask_value(mut self, value: &str) -> Self {
        self.mask_value = Some(value.to_string());
        self
    }

    pub fn with_salt(mut self, salt: &str) -> Self {
        self.hash_salt = Some(salt.to_string());
        self
    }

    pub fn effective_mask_value(&self) -> &str {
        self.mask_value.as_deref().unwrap_or("***")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn classification_json_roundtrip() {
        let cls = Classification::new(vec![
            json!({"sensitivity": "secret", "type": "ssn"}),
            json!({"trust": "internal"}),
        ]);
        let json_str = cls.to_json_string().unwrap();
        let parsed = Classification::from_json_string(&json_str).unwrap();
        assert_eq!(cls, parsed);
    }

    #[test]
    fn classification_empty() {
        let cls = Classification::empty();
        assert!(cls.is_empty());
        assert_eq!(cls.labels().len(), 0);
    }

    #[test]
    fn classification_union_deduplicates() {
        let a = Classification::new(vec![json!({"x": 1}), json!({"y": 2})]);
        let b = Classification::new(vec![json!({"y": 2}), json!({"z": 3})]);
        let merged = a.union(&b);
        assert_eq!(merged.labels().len(), 3);
        assert!(merged.labels().contains(&json!({"x": 1})));
        assert!(merged.labels().contains(&json!({"y": 2})));
        assert!(merged.labels().contains(&json!({"z": 3})));
    }

    #[test]
    fn column_classifications_project() {
        let mut cc = ColumnClassifications::new();
        cc.set(
            "ssn",
            Classification::new(vec![json!({"sensitivity": "secret"})]),
        );
        cc.set(
            "name",
            Classification::new(vec![json!({"sensitivity": "pii"})]),
        );
        cc.set(
            "age",
            Classification::new(vec![json!({"sensitivity": "low"})]),
        );

        let projected = cc.project(&["ssn".into(), "name".into()]);
        assert!(projected.get("ssn").is_some());
        assert!(projected.get("name").is_some());
        assert!(projected.get("age").is_none());
    }

    #[test]
    fn column_classifications_union_all() {
        let mut cc = ColumnClassifications::new();
        cc.set("a", Classification::new(vec![json!({"x": 1})]));
        cc.set("b", Classification::new(vec![json!({"y": 2})]));

        let merged = cc.union_all();
        assert_eq!(merged.labels().len(), 2);
    }

    #[test]
    fn mask_spec_defaults() {
        let spec = MaskSpec::mask("ssn");
        assert_eq!(spec.effective_mask_value(), "***");
        assert_eq!(spec.operation, MaskOperation::Mask);
    }

    #[test]
    fn mask_spec_with_custom_value() {
        let spec = MaskSpec::mask("ssn").with_mask_value("REDACTED");
        assert_eq!(spec.effective_mask_value(), "REDACTED");
    }

    #[test]
    fn classification_serde_roundtrip() {
        let cls = Classification::new(vec![json!({"sensitivity": "secret"})]);
        let serialized = serde_json::to_string(&cls).unwrap();
        let deserialized: Classification = serde_json::from_str(&serialized).unwrap();
        assert_eq!(cls, deserialized);
    }
}
