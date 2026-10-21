use serde::{Deserialize, Serialize};

/// Where execution is, in the document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub id: i64,
    pub name: String,
    /// 0-based document line, when this frame is in a file the document
    /// generated. `None` for a library frame, which is shown read-only rather
    /// than pretended to be part of the document.
    pub line: Option<u32>,
    /// The file as the adapter named it, for frames outside the document.
    pub source: Option<String>,
    /// 0-based line in `source`, after mapping to a contributor document when
    /// possible, otherwise the adapter's raw source line.
    pub source_line: Option<u32>,
    pub in_document: bool,
    /// Raw adapter coordinates, before document mapping; private capture binds
    /// these independently from the human-facing document coordinates.
    #[serde(default)]
    pub adapter_source: Option<String>,
    #[serde(default)]
    pub adapter_line: Option<u32>,
}
