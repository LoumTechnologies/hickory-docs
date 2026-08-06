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

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use hick_exec::node::Node;
use hick_exec::state::MultiDocumentState;
use hick_lang::HickTag;

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

    /// Optional reference to the tag registry for recursive child processing.
    pub registry: Option<&'a TagRegistry>,

    /// Optional extension context for passing opaque handles (e.g. executor).
    pub context: Option<&'a hick_exec::node::Context>,

    /// Source file name for provenance tracking.
    pub source_file: Option<Arc<str>>,
}

impl<'a> ProcessingContext<'a> {
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

// ---------------------------------------------------------------------------
// Exec output control
// ---------------------------------------------------------------------------

/// Controls which parts of an exec transcript are rendered in file output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecShow {
    /// Show commands and output (default).
    All,
    /// Show commands only.
    Command,
    /// Show output only.
    Output,
    /// Show nothing (side-effect only).
    None,
}

/// Parse the `show` attribute of an `<exec>` tag into an `ExecShow`.
pub fn parse_exec_show(tag: &HickTag) -> ExecShow {
    match tag_attr(tag, "show").as_deref() {
        Some("command") => ExecShow::Command,
        Some("output") => ExecShow::Output,
        Some("none") => ExecShow::None,
        _ => ExecShow::All,
    }
}

/// Render transcript entries according to the `show` mode.
pub fn render_transcript(entries: &[TranscriptEntry], show: ExecShow) -> String {
    let mut out = String::new();
    for entry in entries {
        if matches!(show, ExecShow::All | ExecShow::Command) {
            for cmd in &entry.commands {
                for (i, line) in cmd.lines().enumerate() {
                    if i == 0 {
                        out.push_str(&format!("$ {line}\n"));
                    } else {
                        out.push_str(&format!("  {line}\n"));
                    }
                }
            }
        }
        if matches!(show, ExecShow::All | ExecShow::Output) && !entry.output.is_empty() {
            out.push_str(&entry.output);
            out.push('\n');
        }
    }
    out
}

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

impl Default for TagRegistry {
    fn default() -> Self {
        Self::new()
    }
}

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
