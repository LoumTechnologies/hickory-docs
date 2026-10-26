<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-grove/src/namespace.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6836ac0636e6"></a>
## `crates/hick-grove/src/namespace.rs`

<a id="f-6836ac0636e6-NamespaceUri"></a>
### `NamespaceUri`

<storycode:copy id="f-6836ac0636e6-NamespaceUri">
use std::fmt;

use serde::{Deserialize, Serialize};

/// A namespace URI, e.g. `https://example.com/vocab/task#`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NamespaceUri(pub String);

</storycode:copy>
<a id="f-6836ac0636e6-impl-NamespaceUri"></a>
### `impl-NamespaceUri`

<storycode:copy id="f-6836ac0636e6-impl-NamespaceUri">
impl NamespaceUri {
    pub fn new(uri: impl Into<String>) -> Self {
        Self(uri.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

</storycode:copy>
<a id="f-6836ac0636e6-impl-fmt-Display-for-NamespaceUri"></a>
### `impl-fmt::Display for NamespaceUri`

<storycode:copy id="f-6836ac0636e6-impl-fmt-Display-for-NamespaceUri">
impl fmt::Display for NamespaceUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

</storycode:copy>
<a id="f-6836ac0636e6-QualifiedName"></a>
### `QualifiedName`

A namespace-qualified element or attribute name.

<storycode:copy id="f-6836ac0636e6-QualifiedName">
/// A namespace-qualified element or attribute name.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct QualifiedName {
    pub namespace_uri: NamespaceUri,
    pub local_name: String,
}

</storycode:copy>
<a id="f-6836ac0636e6-impl-fmt-Display-for-QualifiedName"></a>
### `impl-fmt::Display for QualifiedName`

<storycode:copy id="f-6836ac0636e6-impl-fmt-Display-for-QualifiedName">
impl fmt::Display for QualifiedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{}}}:{}", self.namespace_uri, self.local_name)
    }
}

</storycode:copy>
<a id="f-6836ac0636e6-DocId"></a>
### `DocId`

Unique identifier for a document within the engine.

<storycode:copy id="f-6836ac0636e6-DocId">
/// Unique identifier for a document within the engine.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocId(pub String);

</storycode:copy>
<a id="f-6836ac0636e6-impl-Default-for-DocId"></a>
### `impl-Default for DocId`

<storycode:copy id="f-6836ac0636e6-impl-Default-for-DocId">
impl Default for DocId {
    fn default() -> Self {
        Self::new()
    }
}

</storycode:copy>
<a id="f-6836ac0636e6-impl-DocId"></a>
### `impl-DocId`

<storycode:copy id="f-6836ac0636e6-impl-DocId">
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

</storycode:copy>
<a id="f-6836ac0636e6-impl-fmt-Display-for-DocId"></a>
### `impl-fmt::Display for DocId`

<storycode:copy id="f-6836ac0636e6-impl-fmt-Display-for-DocId">
impl fmt::Display for DocId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
</storycode:copy>

### Assemble `crates/hick-grove/src/namespace.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-grove/src/namespace.rs"><storycode:paste select="#f-6836ac0636e6-NamespaceUri"/><storycode:paste select="#f-6836ac0636e6-impl-NamespaceUri"/><storycode:paste select="#f-6836ac0636e6-impl-fmt-Display-for-NamespaceUri"/><storycode:paste select="#f-6836ac0636e6-QualifiedName"/><storycode:paste select="#f-6836ac0636e6-impl-fmt-Display-for-QualifiedName"/><storycode:paste select="#f-6836ac0636e6-DocId"/><storycode:paste select="#f-6836ac0636e6-impl-Default-for-DocId"/><storycode:paste select="#f-6836ac0636e6-impl-DocId"/><storycode:paste select="#f-6836ac0636e6-impl-fmt-Display-for-DocId"/></storycode:file>

</storycode:doc>
