<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Included Span Names Its Own File, And An Edit Through It Lands There

For: Authors editing generated text and reviewers tracing a result to its source.

Given a document that splices text from other files (`<hick:include>`,
`<hick:upstream>`), when provenance is built for any output those spans feed,
then each entry's `doc_path` names the file whose bytes the span offsets
actually index — the included file, canonically — and when an edit in a
generated output maps onto such a span, then the edit is applied to that
file, byte-exactly, while the including document is untouched.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/included-spans-name-their-own-file.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-4736a5813d3c"></a>
## `crates/hick-handlers/src/lib.rs`

Related contracts: [An Included Span Names Its Own File, And An Edit Through It Lands There](contract.md).

<a id="f-4736a5813d3c-handlers"></a>
### `handlers`

Extensible tag handler system for hick pipelines.
This crate provides a plugin-like architecture for processing hick XML tags.
Each tag type (copy, paste, exec, etc.) can be implemented as a `TagHandler`
and registered with a `TagRegistry`.
# Example
\```rust,ignore
use hick_handlers::{TagHandler, TagRegistry, TagResult, ProcessingContext};
struct MyCustomHandler;
impl TagHandler for MyCustomHandler {
fn tag_name(&self) -> &str { "my-custom" }
fn phase(&self) -> ProcessingPhase { ProcessingPhase::Content }
fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> anyhow::Result<TagResult> {
// Custom processing logic
Ok(TagResult::Declaration)
}
}
let mut registry = TagRegistry::new();
registry.register(Box::new(MyCustomHandler));
\```

<storycode:copy id="f-4736a5813d3c-handlers">
//! Extensible tag handler system for hick pipelines.
//!
//! This crate provides a plugin-like architecture for processing hick XML tags.
//! Each tag type (copy, paste, exec, etc.) can be implemented as a `TagHandler`
//! and registered with a `TagRegistry`.
//!
//! # Example
//!
//! ```rust,ignore
//! use hick_handlers::{TagHandler, TagRegistry, TagResult, ProcessingContext};
//!
//! struct MyCustomHandler;
//!
//! impl TagHandler for MyCustomHandler {
//!     fn tag_name(&self) -> &str { "my-custom" }
//!     fn phase(&self) -> ProcessingPhase { ProcessingPhase::Content }
//!
//!     fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> anyhow::Result<TagResult> {
//!         // Custom processing logic
//!         Ok(TagResult::Declaration)
//!     }
//! }
//!
//! let mut registry = TagRegistry::new();
//! registry.register(Box::new(MyCustomHandler));
//! ```

pub mod handlers;

</storycode:copy>
<a id="f-4736a5813d3c-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-4736a5813d3c-use_declaration">
use std::collections::HashMap;
</storycode:copy>
<a id="f-4736a5813d3c-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-4736a5813d3c-use_declaration-2">
use std::sync::Arc;

</storycode:copy>
<a id="f-4736a5813d3c-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-4736a5813d3c-use_declaration-3">
use anyhow::Result;
</storycode:copy>
<a id="f-4736a5813d3c-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-4736a5813d3c-use_declaration-4">
use hick_exec::node::Node;
</storycode:copy>
<a id="f-4736a5813d3c-use_declaration-5"></a>
### `use_declaration`

<storycode:copy id="f-4736a5813d3c-use_declaration-5">
use hick_exec::state::MultiDocumentState;
</storycode:copy>
<a id="f-4736a5813d3c-use_declaration-6"></a>
### `use_declaration`

<storycode:copy id="f-4736a5813d3c-use_declaration-6">
use hick_lang::HickTag;

</storycode:copy>
<a id="f-4736a5813d3c-TagHandler"></a>
### `TagHandler`

Tag Handler Trait
Trait for implementing custom tag handlers.
Tag handlers process hick XML elements and can produce various results:
- Output nodes for file content
- Side-effect declarations (variables, substitutions, etc.)
- Multiple nodes

<storycode:copy id="f-4736a5813d3c-TagHandler">
// ---------------------------------------------------------------------------
// Tag Handler Trait
// ---------------------------------------------------------------------------

/// Trait for implementing custom tag handlers.
///
/// Tag handlers process hick XML elements and can produce various results:
/// - Output nodes for file content
/// - Side-effect declarations (variables, substitutions, etc.)
/// - Multiple nodes
pub trait TagHandler: Send + Sync {
    /// The tag name this handler processes (without namespace prefix).
    fn tag_name(&self) -> &str;

    /// The processing phase this handler runs in.
    fn phase(&self) -> ProcessingPhase;

    /// Process the tag and return a result.
    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult>;
}

</storycode:copy>
<a id="f-4736a5813d3c-ProcessingPhase"></a>
### `ProcessingPhase`

Processing Phases
Processing phases for tag handling.
Tags are processed in phase order:
1. Declaration - Register copy/cut/substitute/exclude (side-effect only)
2. Content - Produce file output nodes (exec, paste, val)

<storycode:copy id="f-4736a5813d3c-ProcessingPhase">
// ---------------------------------------------------------------------------
// Processing Phases
// ---------------------------------------------------------------------------

/// Processing phases for tag handling.
///
/// Tags are processed in phase order:
/// 1. Declaration - Register copy/cut/substitute/exclude (side-effect only)
/// 2. Content - Produce file output nodes (exec, paste, val)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProcessingPhase {
    /// Declaration processing (copy, cut, substitute, exclude).
    Declaration = 0,
    /// Content processing (exec, paste, val).
    Content = 1,
}

</storycode:copy>
<a id="f-4736a5813d3c-TagResult"></a>
### `TagResult`

<storycode:copy id="f-4736a5813d3c-TagResult">
// ---------------------------------------------------------------------------
// Tag Result
// ---------------------------------------------------------------------------

/// Result of processing a tag.
#[derive(Default)]
pub enum TagResult {
    /// Produces a single output node.
    Node(Arc<dyn Node>),

    /// Side-effect only (variable declaration, etc.). No output.
    #[default]
    Declaration,

    /// Produces multiple output nodes.
    Nodes(Vec<Arc<dyn Node>>),
}

</storycode:copy>
<a id="f-4736a5813d3c-ProcessingContext"></a>
### `ProcessingContext`

Processing Context
Context provided to tag handlers during processing.

<storycode:copy id="f-4736a5813d3c-ProcessingContext">
// ---------------------------------------------------------------------------
// Processing Context
// ---------------------------------------------------------------------------

/// Context provided to tag handlers during processing.
pub struct ProcessingContext<'a> {
    /// Shared state across documents.
    pub state: &'a Arc<MultiDocumentState>,

    /// Execution transcripts for containers (container name -> entries).
    pub transcripts: &'a HashMap<String, Vec<TranscriptEntry>>,

    /// Number of leading spaces to strip when dedenting content.
    pub indent: usize,

    /// Whitespace before a paste tag that occupies an otherwise empty line.
    /// The file assembler supplies this only for that structural case, so an
    /// inline paste remains a verbatim splice.
    pub paste_line_indent: Option<String>,

    /// Optional reference to the tag registry for recursive child processing.
    pub registry: Option<&'a TagRegistry>,

    /// Optional extension context for passing opaque handles (e.g. executor).
    pub context: Option<&'a hick_exec::node::Context>,

    /// Source file name for provenance tracking.
    pub source_file: Option<Arc<str>>,

    /// The document's span-file table ([`hick_lang::HickDocument::span_files`],
    /// pre-converted): a span stamped with `file_id: Some(i)` holds offsets
    /// into `span_files[i]`, not into `source_file`. Empty when the document
    /// includes nothing.
    pub span_files: &'a [Arc<str>],
}

</storycode:copy>
<a id="f-4736a5813d3c-impl-ProcessingContext-a"></a>
### `impl-ProcessingContext<'a>`

<storycode:copy id="f-4736a5813d3c-impl-ProcessingContext-a">
impl<'a> ProcessingContext<'a> {
    /// The file a span's byte offsets actually index: the spliced file it
    /// was stamped with, else this context's own document. Attributing a
    /// spliced span to the including document is how a reverse edit lands
    /// in the wrong file — every provenance origin must go through here.
    pub fn file_of_span(&self, span: &hick_lang::SourceSpan) -> Option<Arc<str>> {
        if let Some(id) = span.file_id
            && let Some(path) = self.span_files.get(usize::from(id))
        {
            return Some(path.clone());
        }
        self.source_file.clone()
    }

    /// Process a child tag through the registry, returning the first Node result.
    pub fn process_child(&self, child_tag: &HickTag) -> Option<Arc<dyn Node>> {
        let registry = self.registry?;
        let handler = registry.find(&child_tag.name)?;
        match handler.process(child_tag, self) {
            Ok(TagResult::Node(n)) => Some(n),
            _ => None,
        }
    }

    /// Process all child tags, returning the first one that produces a Node.
    pub fn first_child_node(&self, tag: &HickTag) -> Option<Arc<dyn Node>> {
        use hick_lang::HickNode;
        for child in &tag.children {
            if let HickNode::Tag(child_tag) = child
                && let Some(node) = self.process_child(child_tag)
            {
                return Some(node);
            }
        }
        None
    }
}

</storycode:copy>
<a id="f-4736a5813d3c-TranscriptEntry"></a>
### `TranscriptEntry`

A transcript entry for container execution.

<storycode:copy id="f-4736a5813d3c-TranscriptEntry">
/// A transcript entry for container execution.
#[derive(Debug, Clone, Default)]
pub struct TranscriptEntry {
    /// Commands that were executed.
    pub commands: Vec<String>,
    /// Output from the commands.
    pub output: String,
    /// Source line of the `<hick:exec>` tag that produced this entry, when
    /// known. Lets the exec handler render only its own entry instead of the
    /// container's whole transcript.
    pub source_line: Option<usize>,
}

</storycode:copy>
<a id="f-4736a5813d3c-use_declaration-7"></a>
### `use_declaration`

<storycode:copy id="f-4736a5813d3c-use_declaration-7">
// ---------------------------------------------------------------------------
// Exec output control
// ---------------------------------------------------------------------------

pub use hick_lang::ExecShow;

</storycode:copy>
<a id="f-4736a5813d3c-parse_exec_show"></a>
### `parse_exec_show`

Parse the `show` attribute of an `<exec>` tag into an `ExecShow`.

<storycode:copy id="f-4736a5813d3c-parse_exec_show">
/// Parse the `show` attribute of an `<exec>` tag into an `ExecShow`.
pub fn parse_exec_show(tag: &HickTag) -> ExecShow {
    match tag_attr(tag, "show").as_deref() {
        Some("command") => ExecShow::Command,
        Some("output") => ExecShow::Output,
        Some("none") => ExecShow::None,
        _ => ExecShow::All,
    }
}

</storycode:copy>
<a id="f-4736a5813d3c-render_transcript"></a>
### `render_transcript`

Render transcript entries according to the `show` mode.

<storycode:copy id="f-4736a5813d3c-render_transcript">
/// Render transcript entries according to the `show` mode.
pub fn render_transcript(entries: &[TranscriptEntry], show: ExecShow) -> String {
    entries
        .iter()
        .map(|entry| hick_lang::render_transcript_entry(&entry.commands, &entry.output, show))
        .collect()
}
</storycode:copy>
<a id="f-4736a5813d3c-TagRegistry"></a>
### `TagRegistry`

Tag Registry
Registry for tag handlers.
Handlers are stored by name and can be looked up during tag processing.

<storycode:copy id="f-4736a5813d3c-TagRegistry">
// ---------------------------------------------------------------------------
// Tag Registry
// ---------------------------------------------------------------------------

/// Registry for tag handlers.
///
/// Handlers are stored by name and can be looked up during tag processing.
pub struct TagRegistry {
    handlers: Vec<Box<dyn TagHandler>>,
    by_name: HashMap<String, Vec<usize>>,
}

</storycode:copy>
<a id="f-4736a5813d3c-impl-Default-for-TagRegistry"></a>
### `impl-Default for TagRegistry`

<storycode:copy id="f-4736a5813d3c-impl-Default-for-TagRegistry">
impl Default for TagRegistry {
    fn default() -> Self {
        Self::new()
    }
}

</storycode:copy>
<a id="f-4736a5813d3c-impl-TagRegistry"></a>
### `impl-TagRegistry`

<storycode:copy id="f-4736a5813d3c-impl-TagRegistry">
impl TagRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            by_name: HashMap::new(),
        }
    }

    /// Register a handler.
    pub fn register(&mut self, handler: Box<dyn TagHandler>) {
        let name = handler.tag_name().to_string();
        let idx = self.handlers.len();
        self.handlers.push(handler);
        self.by_name.entry(name).or_default().push(idx);
    }

    /// Find the handler for a tag name.
    pub fn find(&self, name: &str) -> Option<&dyn TagHandler> {
        self.by_name
            .get(name)
            .and_then(|indices| indices.first())
            .map(|&idx| self.handlers[idx].as_ref())
    }

    /// Check if a handler is registered for a name.
    pub fn has_handler(&self, name: &str) -> bool {
        self.by_name.contains_key(name)
    }
}

</storycode:copy>
<a id="f-4736a5813d3c-tag_attr"></a>
### `tag_attr`

Helper function for tag attributes
Get an attribute value from a tag by name.

<storycode:copy id="f-4736a5813d3c-tag_attr">
// ---------------------------------------------------------------------------
// Helper function for tag attributes
// ---------------------------------------------------------------------------

/// Get an attribute value from a tag by name.
pub fn tag_attr(tag: &HickTag, name: &str) -> Option<String> {
    tag.attributes
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.clone())
}

</storycode:copy>
<a id="f-4736a5813d3c-origin_key_in"></a>
### `origin_key_in`

Where a tag was declared, as a key that is stable across passes.
`(file, byte offset)` addresses one declaration exactly. The file has to be
RESOLVED, not the raw `span.file_id`: `None` there means "the document
being parsed", which is a different document for each source in a
multi-document pipeline — so two copies at the same offset in two files
shared one key and the second silently replaced the first.
A tag with no span (one a handler synthesised) has no identity to key on
and returns `None`, which registers it the old way: always pushed, never
replaced.

<storycode:copy id="f-4736a5813d3c-origin_key_in">
/// Where a tag was declared, as a key that is stable across passes.
///
/// `(file, byte offset)` addresses one declaration exactly. The file has to be
/// RESOLVED, not the raw `span.file_id`: `None` there means "the document
/// being parsed", which is a different document for each source in a
/// multi-document pipeline — so two copies at the same offset in two files
/// shared one key and the second silently replaced the first.
///
/// A tag with no span (one a handler synthesised) has no identity to key on
/// and returns `None`, which registers it the old way: always pushed, never
/// replaced.
pub fn origin_key_in(tag: &HickTag, file: &str) -> Option<String> {
    let span = tag.source_span?;
    Some(format!("{file}:{}", span.start))
}

</storycode:copy>
<a id="f-4736a5813d3c-origin_key"></a>
### `origin_key`

[`origin_key_in`] for a tag being processed by a handler, whose context
knows which file the span's offsets index.

<storycode:copy id="f-4736a5813d3c-origin_key">
/// [`origin_key_in`] for a tag being processed by a handler, whose context
/// knows which file the span's offsets index.
pub fn origin_key(tag: &HickTag, ctx: &ProcessingContext) -> Option<String> {
    let span = tag.source_span?;
    let file = ctx
        .file_of_span(&span)
        .map(|f| f.to_string())
        .unwrap_or_default();
    Some(format!("{file}:{}", span.start))
}

</storycode:copy>
<a id="f-4736a5813d3c-has_flag"></a>
### `has_flag`

Whether a boolean attribute is set on a tag.
True for the bare flag (`distinct`), for `distinct=""`, and for the
spellings a person reaches for anyway (`"true"`, `"yes"`, the attribute's
own name). `distinct="false"` is respected rather than read as presence,
because a flag that ignores the word `false` is a trap.

<storycode:copy id="f-4736a5813d3c-has_flag">
/// Whether a boolean attribute is set on a tag.
///
/// True for the bare flag (`distinct`), for `distinct=""`, and for the
/// spellings a person reaches for anyway (`"true"`, `"yes"`, the attribute's
/// own name). `distinct="false"` is respected rather than read as presence,
/// because a flag that ignores the word `false` is a trap.
pub fn has_flag(tag: &HickTag, name: &str) -> bool {
    match tag_attr(tag, name) {
        None => false,
        Some(value) => {
            let value = value.trim();
            value.is_empty()
                || value.eq_ignore_ascii_case("true")
                || value.eq_ignore_ascii_case("yes")
                || value.eq_ignore_ascii_case(name)
        }
    }
}

</storycode:copy>
<a id="f-4736a5813d3c-collect_text_children"></a>
### `collect_text_children`

Collect text content from a tag's children into a single string.

<storycode:copy id="f-4736a5813d3c-collect_text_children">
/// Collect text content from a tag's children into a single string.
pub fn collect_text_children(tag: &HickTag) -> String {
    use hick_lang::HickNode;
    let mut result = String::new();
    for child in &tag.children {
        if let HickNode::Text(text, _) = child {
            result.push_str(text);
        }
    }
    result
}

</storycode:copy>
<a id="f-4736a5813d3c-tests"></a>
### `tests`

<storycode:copy id="f-4736a5813d3c-tests">
// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHandler {
        tag_name: String,
    }

    impl TagHandler for TestHandler {
        fn tag_name(&self) -> &str {
            &self.tag_name
        }

        fn phase(&self) -> ProcessingPhase {
            ProcessingPhase::Declaration
        }

        fn process(&self, _tag: &HickTag, _ctx: &ProcessingContext) -> Result<TagResult> {
            Ok(TagResult::Declaration)
        }
    }

    #[test]
    fn registry_finds_handler() {
        let mut registry = TagRegistry::new();
        registry.register(Box::new(TestHandler {
            tag_name: "test".to_string(),
        }));

        assert!(registry.find("test").is_some());
        assert!(registry.find("nonexistent").is_none());
    }

    #[test]
    fn registry_has_handler() {
        let mut registry = TagRegistry::new();
        registry.register(Box::new(TestHandler {
            tag_name: "copy".to_string(),
        }));

        assert!(registry.has_handler("copy"));
        assert!(!registry.has_handler("paste"));
    }
}
</storycode:copy>

### Assemble `crates/hick-handlers/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-handlers/src/lib.rs"><storycode:paste select="#f-4736a5813d3c-handlers"/><storycode:paste select="#f-4736a5813d3c-use_declaration"/><storycode:paste select="#f-4736a5813d3c-use_declaration-2"/><storycode:paste select="#f-4736a5813d3c-use_declaration-3"/><storycode:paste select="#f-4736a5813d3c-use_declaration-4"/><storycode:paste select="#f-4736a5813d3c-use_declaration-5"/><storycode:paste select="#f-4736a5813d3c-use_declaration-6"/><storycode:paste select="#f-4736a5813d3c-TagHandler"/><storycode:paste select="#f-4736a5813d3c-ProcessingPhase"/><storycode:paste select="#f-4736a5813d3c-TagResult"/><storycode:paste select="#f-4736a5813d3c-ProcessingContext"/><storycode:paste select="#f-4736a5813d3c-impl-ProcessingContext-a"/><storycode:paste select="#f-4736a5813d3c-TranscriptEntry"/><storycode:paste select="#f-4736a5813d3c-use_declaration-7"/><storycode:paste select="#f-4736a5813d3c-parse_exec_show"/><storycode:paste select="#f-4736a5813d3c-render_transcript"/><storycode:paste select="#f-4736a5813d3c-TagRegistry"/><storycode:paste select="#f-4736a5813d3c-impl-Default-for-TagRegistry"/><storycode:paste select="#f-4736a5813d3c-impl-TagRegistry"/><storycode:paste select="#f-4736a5813d3c-tag_attr"/><storycode:paste select="#f-4736a5813d3c-origin_key_in"/><storycode:paste select="#f-4736a5813d3c-origin_key"/><storycode:paste select="#f-4736a5813d3c-has_flag"/><storycode:paste select="#f-4736a5813d3c-collect_text_children"/><storycode:paste select="#f-4736a5813d3c-tests"/></storycode:file>

<a id="output-9105369440f8"></a>
## `crates/hick-lang/tests/upstream_tests.rs`

Turn declared document constructions into immutable project trees, retaining byte origins so generated edits can return to their source.

Related contracts: [An Included Span Names Its Own File, And An Edit Through It Lands There](contract.md).

<a id="f-9105369440f8-write"></a>
### `write`

`<hick:upstream>`: a document declares its own place in a pipeline, and
selectors reach anything upstream of it.
The edge carries FRAGMENTS, transitively, and renders nothing — which is
what separates it from `hick:include`, whose job is composing a manual out
of chapters and which brings the prose along on purpose.

<storycode:copy id="f-9105369440f8-write">
//! `<hick:upstream>`: a document declares its own place in a pipeline, and
//! selectors reach anything upstream of it.
//!
//! The edge carries FRAGMENTS, transitively, and renders nothing — which is
//! what separates it from `hick:include`, whose job is composing a manual out
//! of chapters and which brings the prose along on purpose.

use std::collections::HashSet;
use std::path::Path;

use hick_lang::{parse, resolve_includes};

fn write(dir: &Path, name: &str, body: &str) {
    std::fs::write(
        dir.join(name),
        format!("<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n{body}\n</hick:doc>\n"),
    )
    .unwrap();
}

</storycode:copy>
<a id="f-9105369440f8-resolve"></a>
### `resolve`

<storycode:copy id="f-9105369440f8-resolve">
fn resolve(dir: &Path, body: &str) -> Result<hick_lang::HickDocument, hick_lang::ParseError> {
    let mut doc = parse(&format!(
        "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n{body}\n</hick:doc>\n"
    ))?;
    resolve_includes(&mut doc, dir, &mut HashSet::new())?;
    Ok(doc)
}

</storycode:copy>
<a id="f-9105369440f8-chain"></a>
### `chain`

notes → domain → requirements, the shape the real chain has.

<storycode:copy id="f-9105369440f8-chain">
/// notes → domain → requirements, the shape the real chain has.
fn chain(dir: &Path) {
    write(
        dir,
        "notes.md",
        r#"Long meeting preamble nobody downstream should read.
<hick:copy id="metering-basis" class="decision">Meter execution minutes.</hick:copy>
<hick:copy id="hard-stop" class="decision">Refuse, do not queue.</hick:copy>"#,
    );
    write(
        dir,
        "domain.md",
        r#"<hick:upstream file="notes.md" />
<hick:copy id="term-allowance" class="glossary">Minutes per month.</hick:copy>"#,
    );
}

</storycode:copy>
<a id="f-9105369440f8-selectors_reach_through_the_whole_chain_not_just_one_hop"></a>
### `selectors_reach_through_the_whole_chain_not_just_one_hop`

<storycode:copy id="f-9105369440f8-selectors_reach_through_the_whole_chain_not_just_one_hop">
#[test]
fn selectors_reach_through_the_whole_chain_not_just_one_hop() {
    let dir = tempfile::tempdir().unwrap();
    chain(dir.path());
    let doc = resolve(dir.path(), r#"<hick:upstream file="domain.md" />"#).unwrap();
    // The edge stays in the tree holding what it brought, so selectors — not
    // a top-level scan — are how anything reaches it.
    // `notes.md` is two hops away and never named by this document.
    assert_eq!(
        hick_lang::fragments_matching(&doc, "#metering-basis").len(),
        1
    );
    assert_eq!(
        hick_lang::fragments_matching(&doc, "#term-allowance").len(),
        1
    );
    assert_eq!(
        doc.find_tags("upstream").len(),
        1,
        "the edge itself remains"
    );
}

</storycode:copy>
<a id="f-9105369440f8-an_upstream_edge_carries_no_prose"></a>
### `an_upstream_edge_carries_no_prose`

<storycode:copy id="f-9105369440f8-an_upstream_edge_carries_no_prose">
#[test]
fn an_upstream_edge_carries_no_prose() {
    let dir = tempfile::tempdir().unwrap();
    chain(dir.path());
    let doc = resolve(dir.path(), r#"<hick:upstream file="domain.md" />"#).unwrap();
    assert!(
        !format!("{doc:?}").contains("Long meeting preamble"),
        "an upstream edge dragged the source document's prose along"
    );
}

</storycode:copy>
<a id="f-9105369440f8-a_class_still_matches_across_documents"></a>
### `a_class_still_matches_across_documents`

<storycode:copy id="f-9105369440f8-a_class_still_matches_across_documents">
#[test]
fn a_class_still_matches_across_documents() {
    // Explicitly multi-match: accumulating decisions from several documents is
    // the point of the chain, so this must NOT be a collision.
    let dir = tempfile::tempdir().unwrap();
    chain(dir.path());
    let doc = resolve(
        dir.path(),
        r#"<hick:upstream file="domain.md" />
<hick:copy id="local" class="decision">And one decided here.</hick:copy>"#,
    )
    .unwrap();
    assert_eq!(hick_lang::fragments_matching(&doc, ".decision").len(), 3);
}

</storycode:copy>
<a id="f-9105369440f8-a_duplicate_id_across_the_pipeline_is_an_error"></a>
### `a_duplicate_id_across_the_pipeline_is_an_error`

<storycode:copy id="f-9105369440f8-a_duplicate_id_across_the_pipeline_is_an_error">
#[test]
fn a_duplicate_id_across_the_pipeline_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    chain(dir.path());
    let err = resolve(
        dir.path(),
        r#"<hick:upstream file="domain.md" />
<hick:copy id="hard-stop" class="decision">A rival definition.</hick:copy>"#,
    )
    .unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("hard-stop"), "{msg}");
    assert!(
        msg.contains("unique"),
        "the error must state the rule: {msg}"
    );
}

</storycode:copy>
<a id="f-9105369440f8-a_diamond_is_normal_and_merges_each_document_once"></a>
### `a_diamond_is_normal_and_merges_each_document_once`

<storycode:copy id="f-9105369440f8-a_diamond_is_normal_and_merges_each_document_once">
#[test]
fn a_diamond_is_normal_and_merges_each_document_once() {
    // requirements → domain → notes, and requirements → notes directly. Every
    // real pipeline grows this shape; merging notes twice would make every id
    // in it collide with itself.
    let dir = tempfile::tempdir().unwrap();
    chain(dir.path());
    let doc = resolve(
        dir.path(),
        r#"<hick:upstream file="domain.md" />
<hick:upstream file="notes.md" />"#,
    )
    .unwrap();
    assert_eq!(hick_lang::fragments_matching(&doc, "#hard-stop").len(), 1);
}

</storycode:copy>
<a id="f-9105369440f8-a_cycle_is_refused_rather_than_followed"></a>
### `a_cycle_is_refused_rather_than_followed`

<storycode:copy id="f-9105369440f8-a_cycle_is_refused_rather_than_followed">
#[test]
fn a_cycle_is_refused_rather_than_followed() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "a.md", r#"<hick:upstream file="b.md" />"#);
    write(dir.path(), "b.md", r#"<hick:upstream file="a.md" />"#);
    let err = resolve(dir.path(), r#"<hick:upstream file="a.md" />"#).unwrap_err();
    assert!(format!("{err}").contains("circular"), "{err}");
}

</storycode:copy>
<a id="f-9105369440f8-a_missing_upstream_names_the_file_it_could_not_read"></a>
### `a_missing_upstream_names_the_file_it_could_not_read`

<storycode:copy id="f-9105369440f8-a_missing_upstream_names_the_file_it_could_not_read">
#[test]
fn a_missing_upstream_names_the_file_it_could_not_read() {
    let dir = tempfile::tempdir().unwrap();
    let err = resolve(dir.path(), r#"<hick:upstream file="nope.md" />"#).unwrap_err();
    assert!(format!("{err}").contains("nope.md"), "{err}");
}

</storycode:copy>
<a id="f-9105369440f8-span_file_attribution"></a>
### `span_file_attribution`

Protects docs/guarantees/lineage/included-spans-name-their-own-file.md

<storycode:copy id="f-9105369440f8-span_file_attribution">
// Protects docs/guarantees/lineage/included-spans-name-their-own-file.md
mod span_file_attribution {
    use super::*;
    use hick_lang::HickNode;

    /// Every span in `nodes`, with the file (from the table) it claims.
    fn spans_with_files(doc: &hick_lang::HickDocument) -> Vec<(Option<String>, usize, usize)> {
        fn walk(
            nodes: &[HickNode],
            table: &[String],
            out: &mut Vec<(Option<String>, usize, usize)>,
        ) {
            for node in nodes {
                match node {
                    HickNode::Text(_, Some(span)) => out.push((
                        span.file_id.map(|id| table[usize::from(id)].clone()),
                        span.start,
                        span.end,
                    )),
                    HickNode::Text(_, None) => {}
                    HickNode::Tag(tag) => walk(&tag.children, table, out),
                }
            }
        }
        let mut out = Vec::new();
        walk(&doc.nodes, &doc.span_files, &mut out);
        out
    }

    #[test]
    fn included_text_spans_name_the_included_file() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "chapter.md",
            "Chapter prose that lives in chapter.md.",
        );
        let doc = resolve(dir.path(), r#"<hick:include file="chapter.md" />"#).unwrap();

        assert_eq!(doc.span_files.len(), 1);
        assert!(doc.span_files[0].ends_with("chapter.md"));

        let spans = spans_with_files(&doc);
        let foreign: Vec<_> = spans.iter().filter(|(f, _, _)| f.is_some()).collect();
        assert!(
            !foreign.is_empty(),
            "included spans must be stamped: {spans:?}"
        );
        for (file, start, end) in &foreign {
            let file = file.as_ref().unwrap();
            assert!(file.ends_with("chapter.md"), "{file}");
            // The offsets must be valid in the file they claim.
            let source = std::fs::read_to_string(file).unwrap();
            assert!(
                *end <= source.len(),
                "span {start}..{end} out of bounds for {file}"
            );
        }
        // The including document's own spans stay unstamped.
        assert!(spans.iter().any(|(f, _, _)| f.is_none()) || spans.len() == foreign.len());
    }

    #[test]
    fn upstream_fragment_spans_name_the_upstream_file() {
        let dir = tempfile::tempdir().unwrap();
        chain(dir.path());
        let doc = resolve(dir.path(), r#"<hick:upstream file="domain.md" />"#).unwrap();
        // notes.md arrives transitively through domain.md; both are in
        // the table, and the #hard-stop fragment's span names notes.md.
        assert!(doc.span_files.iter().any(|f| f.ends_with("domain.md")));
        assert!(doc.span_files.iter().any(|f| f.ends_with("notes.md")));

        let spans = spans_with_files(&doc);
        let hard_stop_text = "Refuse, do not queue.";
        let owning = spans.iter().find(|(f, s, e)| {
            f.as_ref().is_some_and(|f| {
                std::fs::read_to_string(f)
                    .ok()
                    .and_then(|src| src.get(*s..*e).map(|t| t == hard_stop_text))
                    .unwrap_or(false)
            })
        });
        let (file, _, _) = owning.expect("the fragment's span must map byte-exactly to its file");
        assert!(file.as_ref().unwrap().ends_with("notes.md"));
    }

    #[test]
    fn a_document_without_includes_has_an_empty_table() {
        let dir = tempfile::tempdir().unwrap();
        let doc = resolve(dir.path(), "Just prose.").unwrap();
        assert!(doc.span_files.is_empty());
        assert!(spans_with_files(&doc).iter().all(|(f, _, _)| f.is_none()));
    }
}
</storycode:copy>

### Assemble `crates/hick-lang/tests/upstream_tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lang/tests/upstream_tests.rs"><storycode:paste select="#f-9105369440f8-write"/><storycode:paste select="#f-9105369440f8-resolve"/><storycode:paste select="#f-9105369440f8-chain"/><storycode:paste select="#f-9105369440f8-selectors_reach_through_the_whole_chain_not_just_one_hop"/><storycode:paste select="#f-9105369440f8-an_upstream_edge_carries_no_prose"/><storycode:paste select="#f-9105369440f8-a_class_still_matches_across_documents"/><storycode:paste select="#f-9105369440f8-a_duplicate_id_across_the_pipeline_is_an_error"/><storycode:paste select="#f-9105369440f8-a_diamond_is_normal_and_merges_each_document_once"/><storycode:paste select="#f-9105369440f8-a_cycle_is_refused_rather_than_followed"/><storycode:paste select="#f-9105369440f8-a_missing_upstream_names_the_file_it_could_not_read"/><storycode:paste select="#f-9105369440f8-span_file_attribution"/></storycode:file>

<a id="output-08372f0e9f69"></a>
## `crates/hickory-cli/tests/include_lineage.rs`

Related contracts: [An Included Span Names Its Own File, And An Edit Through It Lands There](contract.md).

<a id="f-08372f0e9f69-write_doc"></a>
### `write_doc`

Lineage across `<hick:include>` / `<hick:upstream>`: a span spliced from
another file must be attributed to THAT file, and a reverse edit through
it must land in that file — byte-exactly — while the including document
stays untouched.
Before span-file stamping existed, these spans were attributed to the
including document with offsets into the included one: a reverse edit was
silent corruption of whichever text happened to sit at those offsets.
Protects docs/guarantees/lineage/included-spans-name-their-own-file.md

<storycode:copy id="f-08372f0e9f69-write_doc">
//! Lineage across `<hick:include>` / `<hick:upstream>`: a span spliced from
//! another file must be attributed to THAT file, and a reverse edit through
//! it must land in that file — byte-exactly — while the including document
//! stays untouched.
//!
//! Before span-file stamping existed, these spans were attributed to the
//! including document with offsets into the included one: a reverse edit was
//! silent corruption of whichever text happened to sit at those offsets.
//!
//! Protects docs/guarantees/lineage/included-spans-name-their-own-file.md

use std::collections::HashMap;
use std::path::Path;

use hickory_cli::{DocRun, ExecutorChoice, RunMode, output_lineage, run_doc};
use hickory_lineage::{OutputEdit, apply_source_edits, map_edits};

fn write_doc(dir: &Path, name: &str, body: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(
        &path,
        format!("<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n{body}\n</hick:doc>\n"),
    )
    .unwrap();
    path
}

</storycode:copy>
<a id="f-08372f0e9f69-weave"></a>
### `weave`

<storycode:copy id="f-08372f0e9f69-weave">
async fn weave(doc_path: &Path) -> DocRun {
    run_doc(doc_path, &[], RunMode::Weave, ExecutorChoice::Local)
        .await
        .expect("weave must succeed")
}

</storycode:copy>
<a id="f-08372f0e9f69-output_text"></a>
### `output_text`

<storycode:copy id="f-08372f0e9f69-output_text">
fn output_text(run: &DocRun, path: &str) -> String {
    match run.result.files.get(path).expect("output exists") {
        hick_exec::node::FileContent::Text(s) => s.clone(),
        other => panic!("expected text output, got {other:?}"),
    }
}

</storycode:copy>
<a id="f-08372f0e9f69-an_edit_to_included_content_lands_in_the_included_file"></a>
### `an_edit_to_included_content_lands_in_the_included_file`

An included file's `<hick:file>` content is attributed to the included
file, and an edit routed through the output lands there.

<storycode:copy id="f-08372f0e9f69-an_edit_to_included_content_lands_in_the_included_file">
/// An included file's `<hick:file>` content is attributed to the included
/// file, and an edit routed through the output lands there.
#[tokio::test]
async fn an_edit_to_included_content_lands_in_the_included_file() {
    let dir = tempfile::tempdir().unwrap();
    let chapter = write_doc(
        dir.path(),
        "chapter.md",
        "<hick:file path=\"app.py\">def helper():\n    return 41\n</hick:file>",
    );
    let main = write_doc(
        dir.path(),
        "main.md",
        "Main prose.\n<hick:include file=\"chapter.md\" />",
    );

    let run = weave(&main).await;
    let content = output_text(&run, "app.py");
    assert!(content.contains("return 41"), "{content}");

    let provenance = output_lineage(&run, "app.py").expect("app.py has lineage");

    // Every editable byte of this output must name chapter.md, not
    // main.md — the offsets are chapter.md's.
    let chapter_canonical = chapter.canonicalize().unwrap();
    let chapter_source = std::fs::read_to_string(&chapter).unwrap();
    let mut saw_chapter = false;
    for p in &provenance {
        if let Some((doc, s, e)) = p.origin.source() {
            assert!(
                Path::new(doc) == chapter_canonical,
                "output bytes {}..{} attributed to {doc}, expected {}",
                p.start,
                p.end,
                chapter_canonical.display()
            );
            // Byte-exact: the claimed span holds exactly the output bytes.
            assert_eq!(
                &chapter_source[s..e],
                &content[p.start..p.end],
                "span {s}..{e} of chapter.md does not match output {}..{}",
                p.start,
                p.end
            );
            saw_chapter = true;
        }
    }
    assert!(saw_chapter, "no editable provenance at all: {provenance:?}");

    // Route an edit: 41 -> 42.
    let pos = content.find("41").unwrap();
    let edit = OutputEdit {
        start: pos,
        end: pos + 2,
        text: "42".into(),
    };
    let source_edits = map_edits(&content, std::slice::from_ref(&edit), &provenance).unwrap();
    assert!(
        source_edits
            .iter()
            .all(|e| Path::new(&e.doc_path) == chapter_canonical),
        "{source_edits:?}"
    );

    let mut sources = HashMap::new();
    for e in &source_edits {
        sources
            .entry(e.doc_path.clone())
            .or_insert_with(|| std::fs::read_to_string(&e.doc_path).unwrap());
    }
    let updated = apply_source_edits(&sources, &source_edits).unwrap();
    for (doc_path, new_source) in &updated {
        std::fs::write(doc_path, new_source).unwrap();
    }

    // The included file changed; the including document did not.
    assert!(
        std::fs::read_to_string(&chapter)
            .unwrap()
            .contains("return 42"),
        "edit did not land in chapter.md"
    );
    let main_source = std::fs::read_to_string(&main).unwrap();
    assert!(main_source.contains("Main prose."), "{main_source}");
    assert!(
        !main_source.contains("42"),
        "main.md was corrupted: {main_source}"
    );

    // And the round trip closes: re-weaving reproduces the edited output.
    let rerun = weave(&main).await;
    assert!(output_text(&rerun, "app.py").contains("return 42"));
}

</storycode:copy>
<a id="f-08372f0e9f69-an_edit_to_a_pasted_upstream_fragment_lands_upstream"></a>
### `an_edit_to_a_pasted_upstream_fragment_lands_upstream`

A fragment pasted from an upstream document is attributed to the upstream
file, and editing the pasted bytes rewrites that file.

<storycode:copy id="f-08372f0e9f69-an_edit_to_a_pasted_upstream_fragment_lands_upstream">
/// A fragment pasted from an upstream document is attributed to the upstream
/// file, and editing the pasted bytes rewrites that file.
#[tokio::test]
async fn an_edit_to_a_pasted_upstream_fragment_lands_upstream() {
    let dir = tempfile::tempdir().unwrap();
    let notes = write_doc(
        dir.path(),
        "notes.md",
        "<hick:copy id=\"rule\">Refuse, do not queue.</hick:copy>",
    );
    let main = write_doc(
        dir.path(),
        "main.md",
        "<hick:upstream file=\"notes.md\" />\n<hick:file path=\"rules.txt\">\
         <hick:paste select=\"#rule\" /></hick:file>",
    );

    let run = weave(&main).await;
    let content = output_text(&run, "rules.txt");
    assert!(content.contains("Refuse, do not queue."), "{content}");

    let provenance = output_lineage(&run, "rules.txt").unwrap();
    let notes_canonical = notes.canonicalize().unwrap();

    let pos = content.find("queue").unwrap();
    let edit = OutputEdit {
        start: pos,
        end: pos + "queue".len(),
        text: "wait".into(),
    };
    let source_edits = map_edits(&content, std::slice::from_ref(&edit), &provenance).unwrap();
    assert!(
        source_edits
            .iter()
            .all(|e| Path::new(&e.doc_path) == notes_canonical),
        "expected the edit to route to notes.md: {source_edits:?}"
    );

    let mut sources = HashMap::new();
    for e in &source_edits {
        sources
            .entry(e.doc_path.clone())
            .or_insert_with(|| std::fs::read_to_string(&e.doc_path).unwrap());
    }
    let updated = apply_source_edits(&sources, &source_edits).unwrap();
    for (doc_path, new_source) in &updated {
        std::fs::write(doc_path, new_source).unwrap();
    }
    assert!(
        std::fs::read_to_string(&notes)
            .unwrap()
            .contains("do not wait"),
        "edit did not land upstream"
    );
    assert!(!std::fs::read_to_string(&main).unwrap().contains("wait"));
}

</storycode:copy>
<a id="f-08372f0e9f69-a_mid_character_source_span_is_an_error_not_a_panic"></a>
### `a_mid_character_source_span_is_an_error_not_a_panic`

A stale-provenance span that would split a UTF-8 character is an error,
never a panic (replace_range aborts on a non-boundary offset).

<storycode:copy id="f-08372f0e9f69-a_mid_character_source_span_is_an_error_not_a_panic">
/// A stale-provenance span that would split a UTF-8 character is an error,
/// never a panic (replace_range aborts on a non-boundary offset).
#[test]
fn a_mid_character_source_span_is_an_error_not_a_panic() {
    let mut sources = HashMap::new();
    sources.insert("doc.md".to_string(), "café".to_string());
    let edits = vec![hickory_lineage::SourceEdit {
        doc_path: "doc.md".into(),
        span: (3, 4), // inside the two-byte 'é'
        text: "x".into(),
    }];
    let err = apply_source_edits(&sources, &edits).unwrap_err();
    assert!(format!("{err}").contains("character boundaries"), "{err}");
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/include_lineage.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/include_lineage.rs"><storycode:paste select="#f-08372f0e9f69-write_doc"/><storycode:paste select="#f-08372f0e9f69-weave"/><storycode:paste select="#f-08372f0e9f69-output_text"/><storycode:paste select="#f-08372f0e9f69-an_edit_to_included_content_lands_in_the_included_file"/><storycode:paste select="#f-08372f0e9f69-an_edit_to_a_pasted_upstream_fragment_lands_upstream"/><storycode:paste select="#f-08372f0e9f69-a_mid_character_source_span_is_an_error_not_a_panic"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-handlers/src/handlers/copy.rs](../../../mechanisms/crates/hick-handlers/src/handlers/copy.rs.md#output-2bc5aa33e241)
- [crates/hick-handlers/src/lib.rs](contract.md#output-4736a5813d3c)
- [crates/hick-lang/src/lib.rs](../../../mechanisms/crates/hick-lang/src/lib.rs.md#output-0cf7120750a0)
- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)
- [crates/hick-literate/src/weave.rs](../../../mechanisms/crates/hick-literate/src/weave.rs.md#output-7fd582622cb2)
- [crates/hickory-agent/src/tools/mod.rs](../../../mechanisms/crates/hickory-agent/src/tools/mod.rs.md#output-54c7fa39e3da)
- [crates/hickory-cli/src/up/mod.rs](../../../mechanisms/crates/hickory-cli/src/up/mod.rs.md#output-73148a3ab9db)
- [crates/hickory-lineage/src/lib.rs](../../../mechanisms/crates/hickory-lineage/src/lib.rs.md#output-bc43b0352ff3)

### Tests and fixtures

- [crates/hick-lang/tests/upstream_tests.rs](contract.md#output-9105369440f8)
- [crates/hickory-cli/tests/include_lineage.rs](contract.md#output-08372f0e9f69)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
