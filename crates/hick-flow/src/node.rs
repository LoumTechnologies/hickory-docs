//! Node trait and core node types.
//!
//! - `Node` trait: object-safe computation node
//! - `NodeTrace`: immutable trace through the node tree
//! - `StringNode`: leaf node holding literal text
//! - `SeparatorNode`: marker wrapper for separator detection

use std::any::Any;
use std::io;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use futures::{Stream, StreamExt};
use hick_lang::SourceSpan;

use crate::context::Context;

// ---------------------------------------------------------------------------
// SourceOrigin — provenance metadata
// ---------------------------------------------------------------------------

/// Describes where a node's content originated.
///
/// Used for character-level provenance tracking: mapping generated output
/// back to source `.hick` file positions or other origins.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum SourceOrigin {
    /// Literal text from a `.hick` source file.
    Literal { file: Arc<str>, span: SourceSpan },
    /// Output from a container exec.
    Exec {
        container: Arc<str>,
        tag_line: usize,
    },
    /// Content resolved via a paste selector.
    ///
    /// When the pasted bytes are byte-identical to a contiguous region of a
    /// source `.hick` file (a plain-text copy block, no dedent applied),
    /// `file`/`span` carry that region so edits to the pasted output can be
    /// mapped back to the copy block's source text.
    Paste {
        selector: Arc<str>,
        #[cfg_attr(feature = "serde", serde(default))]
        file: Option<Arc<str>>,
        #[cfg_attr(feature = "serde", serde(default))]
        span: Option<SourceSpan>,
    },
    /// Bytes authored by an agent cell (`hick:agent`).
    ///
    /// `session` names the `hick:session` document that recorded the
    /// reasoning; `turn` is the zero-based turn within it. There is
    /// deliberately **no `author` field**: authorship composes instead —
    /// lineage maps a byte to a document span and `git blame` on that span
    /// gives the commit author, which is anchored in a commit and can be
    /// signed, rather than asserted by whoever ran `promote`.
    ///
    /// Sessions are per-author and may be private, so a reader who cannot
    /// open the session still learns the session id and the turn. See
    /// `docs/guarantees/lineage/agent-lineage-degrades-without-a-session.md`.
    ///
    /// `file`/`span` carry the document region the agent's edit landed in
    /// when the emitted bytes are byte-identical to it (the cell writes
    /// through `edit_doc`, so ordinarily they are). Both default to absent,
    /// so an origin serialized without them still deserializes.
    Agent {
        session: Arc<str>,
        turn: usize,
        #[cfg_attr(feature = "serde", serde(default))]
        file: Option<Arc<str>>,
        #[cfg_attr(feature = "serde", serde(default))]
        span: Option<SourceSpan>,
    },
    /// Bytes a tool outside this document wrote, ingested into it.
    ///
    /// The dual thing `SourceOrigin::Paste` is, for a different reason:
    /// present, byte-precise document text that also names where it came
    /// from. A scaffolder's forty files are neither `Literal` (they are not
    /// text you wrote, and blame must not say they are) nor `Exec` (that is
    /// synthetic, and it would kill the reverse edit on the very bytes you
    /// most want to edit). See
    /// `docs/specs/freeform/owning-what-a-scaffolder-wrote.md`.
    ///
    /// `run` is the fingerprint recorded on the `<hick:ingested>` element —
    /// one fingerprint per run, N files under it, because a scaffold is a
    /// single event that happens to write forty things.
    Ingested {
        file: Arc<str>,
        span: SourceSpan,
        run: Arc<str>,
    },
    /// Value of a variable.
    Variable { name: Arc<str> },
    /// Output from a shell script execution.
    Script { tag_line: usize },
    /// Programmatically constructed, no source.
    Synthetic,
}

/// Type alias for boxed streams.
pub type BoxStream<T> = Pin<Box<dyn Stream<Item = T> + Send + 'static>>;

// ---------------------------------------------------------------------------
// Node trait
// ---------------------------------------------------------------------------

/// Object-safe trait for tree-structured computation nodes.
pub trait Node: Send + Sync {
    /// Primary method: returns a stream of `NodeTrace` vectors.
    fn get_stream(self: Arc<Self>, context: Context) -> BoxStream<Vec<NodeTrace>>;

    /// Type discriminator without downcasting.
    fn is_separator(&self) -> bool {
        false
    }

    /// Extract string value if this is a `StringNode`.
    fn as_string_value(&self) -> Option<&str> {
        None
    }

    /// Typed value for convergence. Defaults to wrapping `as_string_value()`.
    fn node_value(&self) -> NodeValue {
        match self.as_string_value() {
            Some(s) => NodeValue::Text(s.to_string()),
            None => NodeValue::None,
        }
    }

    /// Per-node identifier in bytes. Defaults to address-based.
    fn id_bytes(&self) -> Vec<u8> {
        let addr = (self as *const _ as *const ()).addr() as u128;
        addr.to_le_bytes().to_vec()
    }

    /// Downcast hook: returns `self` as `&dyn Any` for type-based dispatch.
    ///
    /// Override this in concrete node types that need to be identified by
    /// downstream consumers (e.g. `RecordChangeNode` in hick-live).
    /// The default returns `None`, meaning the node cannot be downcast.
    fn as_any(&self) -> Option<&dyn Any> {
        None
    }

    /// Provenance metadata: where this node's content originated.
    ///
    /// Returns `None` for nodes that don't carry provenance (default).
    fn source_origin(&self) -> Option<&SourceOrigin> {
        None
    }
}

// ---------------------------------------------------------------------------
// NodeTrace
// ---------------------------------------------------------------------------

/// Immutable, cloneable trace representing a computation path through nodes.
#[derive(Clone)]
pub struct NodeTrace {
    final_result: Arc<dyn Node>,
    trace: Vec<Arc<dyn Node>>,
}

impl NodeTrace {
    pub fn new(final_result: Arc<dyn Node>) -> Self {
        Self {
            final_result,
            trace: Vec::new(),
        }
    }

    /// Record a passthrough node in the trace without changing the final result.
    pub fn add_to_trace(&self, node: Arc<dyn Node>) -> Self {
        let mut t = self.trace.clone();
        t.push(node);
        Self {
            final_result: self.final_result.clone(),
            trace: t,
        }
    }

    /// Record a transformation: changes the final result and records the
    /// transformer in the trace.
    pub fn transform(&self, new_result: Arc<dyn Node>, transformer: Arc<dyn Node>) -> Self {
        let mut t = self.trace.clone();
        t.push(transformer);
        Self {
            final_result: new_result,
            trace: t,
        }
    }

    pub fn final_result(&self) -> Arc<dyn Node> {
        self.final_result.clone()
    }

    pub fn trace(&self) -> &[Arc<dyn Node>] {
        &self.trace
    }
}

// ---------------------------------------------------------------------------
// NodeValue / BinaryData / FileContent
// ---------------------------------------------------------------------------

/// What a leaf node holds. Used by `converge()` to decide output format.
#[non_exhaustive]
pub enum NodeValue {
    Text(String),
    Binary(BinaryData),
    None,
}

/// Binary payload — either held in memory or spilled to a temp file.
pub enum BinaryData {
    Inline(Vec<u8>),
    TempFile(PathBuf),
}

impl BinaryData {
    /// Materialise the bytes (reads from disk for `TempFile`).
    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        match self {
            BinaryData::Inline(v) => Ok(v.clone()),
            BinaryData::TempFile(p) => std::fs::read(p),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            BinaryData::Inline(v) => v.len(),
            BinaryData::TempFile(p) => std::fs::metadata(p).map(|m| m.len() as usize).unwrap_or(0),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Converged output of a file — text or binary.
pub enum FileContent {
    Text(String),
    Binary(BinaryData),
}

impl FileContent {
    pub fn len(&self) -> usize {
        match self {
            FileContent::Text(s) => s.len(),
            FileContent::Binary(d) => d.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return text content if this is a text file.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            FileContent::Text(s) => Some(s),
            FileContent::Binary(_) => None,
        }
    }

    /// Check if this text content contains a substring. Returns false for binary.
    pub fn contains(&self, pat: &str) -> bool {
        match self {
            FileContent::Text(s) => s.contains(pat),
            FileContent::Binary(_) => false,
        }
    }

    /// Trim whitespace from text content. Returns self for binary.
    pub fn trim(&self) -> &str {
        match self {
            FileContent::Text(s) => s.trim(),
            FileContent::Binary(_) => "",
        }
    }
}

impl PartialEq<str> for FileContent {
    fn eq(&self, other: &str) -> bool {
        match self {
            FileContent::Text(s) => s == other,
            FileContent::Binary(_) => false,
        }
    }
}

impl PartialEq<&str> for FileContent {
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl std::fmt::Display for FileContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileContent::Text(s) => f.write_str(s),
            FileContent::Binary(d) => write!(f, "<binary {} bytes>", d.len()),
        }
    }
}

impl std::fmt::Debug for FileContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileContent::Text(s) => write!(f, "FileContent::Text({:?})", s),
            FileContent::Binary(d) => write!(f, "FileContent::Binary({} bytes)", d.len()),
        }
    }
}

// ---------------------------------------------------------------------------
// BinaryNode
// ---------------------------------------------------------------------------

/// Leaf node holding binary data (images, archives, etc.).
pub struct BinaryNode {
    data: BinaryData,
    mime_type: Option<String>,
    stable_id: Option<Vec<u8>>,
}

impl BinaryNode {
    pub fn new(data: BinaryData) -> Self {
        Self {
            data,
            mime_type: None,
            stable_id: None,
        }
    }

    pub fn new_with_mime(data: BinaryData, mime: impl Into<String>) -> Self {
        Self {
            data,
            mime_type: Some(mime.into()),
            stable_id: None,
        }
    }

    pub fn new_with_id(data: BinaryData, mime: Option<String>, id: Vec<u8>) -> Self {
        Self {
            data,
            mime_type: mime,
            stable_id: Some(id),
        }
    }

    pub fn mime_type(&self) -> Option<&str> {
        self.mime_type.as_deref()
    }

    pub fn data(&self) -> &BinaryData {
        &self.data
    }
}

impl Node for BinaryNode {
    fn node_value(&self) -> NodeValue {
        match &self.data {
            BinaryData::Inline(v) => NodeValue::Binary(BinaryData::Inline(v.clone())),
            BinaryData::TempFile(p) => NodeValue::Binary(BinaryData::TempFile(p.clone())),
        }
    }

    fn get_stream(self: Arc<Self>, _context: Context) -> BoxStream<Vec<NodeTrace>> {
        let item = vec![NodeTrace::new(self as Arc<dyn Node>)];
        Box::pin(futures::stream::once(async move { item }))
    }

    fn id_bytes(&self) -> Vec<u8> {
        if let Some(b) = &self.stable_id {
            return b.clone();
        }
        let addr = (self as *const BinaryNode as *const ()).addr() as u128;
        addr.to_le_bytes().to_vec()
    }
}

// ---------------------------------------------------------------------------
// StringNode
// ---------------------------------------------------------------------------

/// Terminal node representing a string value.
pub struct StringNode {
    value: String,
    stable_id: Option<Vec<u8>>,
}

impl StringNode {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            stable_id: None,
        }
    }

    pub fn new_with_id(value: impl Into<String>, id_bytes: Option<Vec<u8>>) -> Self {
        Self {
            value: value.into(),
            stable_id: id_bytes,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl Node for StringNode {
    fn as_string_value(&self) -> Option<&str> {
        Some(self.value())
    }

    fn node_value(&self) -> NodeValue {
        NodeValue::Text(self.value.clone())
    }

    fn get_stream(self: Arc<Self>, _context: Context) -> BoxStream<Vec<NodeTrace>> {
        let item = vec![NodeTrace::new(self as Arc<dyn Node>)];
        Box::pin(futures::stream::once(async move { item }))
    }

    fn id_bytes(&self) -> Vec<u8> {
        if let Some(b) = &self.stable_id {
            return b.clone();
        }
        let addr = (self as *const StringNode as *const ()).addr() as u128;
        addr.to_le_bytes().to_vec()
    }
}

// ---------------------------------------------------------------------------
// SpanNode — string with provenance
// ---------------------------------------------------------------------------

/// Terminal node holding a string value plus provenance metadata.
///
/// Like `StringNode` but also carries a [`SourceOrigin`] describing where
/// the content came from (literal source span, exec output, paste, etc.).
pub struct SpanNode {
    value: String,
    origin: SourceOrigin,
}

impl SpanNode {
    pub fn new(value: impl Into<String>, origin: SourceOrigin) -> Self {
        Self {
            value: value.into(),
            origin,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn origin(&self) -> &SourceOrigin {
        &self.origin
    }
}

impl Node for SpanNode {
    fn as_string_value(&self) -> Option<&str> {
        Some(&self.value)
    }

    fn node_value(&self) -> NodeValue {
        NodeValue::Text(self.value.clone())
    }

    fn source_origin(&self) -> Option<&SourceOrigin> {
        Some(&self.origin)
    }

    fn get_stream(self: Arc<Self>, _context: Context) -> BoxStream<Vec<NodeTrace>> {
        let item = vec![NodeTrace::new(self as Arc<dyn Node>)];
        Box::pin(futures::stream::once(async move { item }))
    }
}

// ---------------------------------------------------------------------------
// SeparatorNode
// ---------------------------------------------------------------------------

/// Marker node that wraps another node and adds itself to traces for later filtering.
pub struct SeparatorNode {
    value: Arc<dyn Node>,
}

impl SeparatorNode {
    pub fn new(value: Arc<dyn Node>) -> Self {
        Self { value }
    }
}

impl Node for SeparatorNode {
    fn is_separator(&self) -> bool {
        true
    }

    fn get_stream(self: Arc<Self>, context: Context) -> BoxStream<Vec<NodeTrace>> {
        let inner = self.value.clone().get_stream(context);
        let me: Arc<dyn Node> = self;

        Box::pin(inner.map(move |items| {
            items
                .into_iter()
                .map(|trace| trace.add_to_trace(me.clone()))
                .collect::<Vec<_>>()
        }))
    }
}
