use std::fmt;

use serde::{Deserialize, Serialize};

/// A namespace URI, e.g. `https://example.com/vocab/task#`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NamespaceUri(pub String);

impl NamespaceUri {
    pub fn new(uri: impl Into<String>) -> Self {
        Self(uri.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NamespaceUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A namespace-qualified element or attribute name.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct QualifiedName {
    pub namespace_uri: NamespaceUri,
    pub local_name: String,
}

impl fmt::Display for QualifiedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{}}}:{}", self.namespace_uri, self.local_name)
    }
}

/// Unique identifier for a document within the engine.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocId(pub String);

impl Default for DocId {
    fn default() -> Self {
        Self::new()
    }
}

impl DocId {
    pub fn new() -> Self {
        Self(format!("doc-{}", uuid::Uuid::now_v7()))
    }

    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DocId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
