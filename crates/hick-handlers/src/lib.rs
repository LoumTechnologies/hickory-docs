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
//!     fn name(&self) -> &str { "my-custom" }
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
pub mod transform;

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use hick_exec::node::{InsertionPoint, Node};
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
/// - Signals to process children
pub trait TagHandler: Send + Sync {
    /// The tag name this handler processes (without namespace prefix).
    fn name(&self) -> &str;

    /// Process the tag and return a result.
    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult>;

    /// Handler priority (higher runs first). Default is 0.
    fn priority(&self) -> i32 {
        0
    }

    /// Whether this handler produces a transformation of its children.
    fn is_transformation(&self) -> bool {
        false
    }

    /// Optional phase for multi-pass processing.
    /// Handlers are grouped by phase and all handlers in a phase run before
    /// moving to the next phase.
    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Main
    }
}

// ---------------------------------------------------------------------------
// Processing Phases
// ---------------------------------------------------------------------------

/// Processing phases for multi-pass tag handling.
///
/// Tags are processed in phase order:
/// 1. Variables - Scan and register variables
/// 2. Features - Process feature definitions
/// 3. Conditional - Filter based on conditions
/// 4. Declaration - Register copy/cut/substitute/etc.
/// 5. Main - Process file outputs and other content
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProcessingPhase {
    /// Variable scanning (first pass).
    Variables = 0,
    /// Feature registration.
    Features = 1,
    /// Conditional filtering.
    Conditional = 2,
    /// Declaration processing (copy, cut, substitute, etc.).
    Declaration = 3,
    /// Main content processing (file, exec, etc.).
    Main = 4,
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

    /// Signal to recursively process children.
    ProcessChildren,

    /// Signal to skip this tag entirely (filtered by condition).
    Skip,

    /// Signal to unwrap children (promote to parent level).
    UnwrapChildren,
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

    /// The parent insertion point (for adding output).
    pub parent_ip: Option<&'a Arc<InsertionPoint>>,

    /// Current document name.
    pub document_name: &'a str,
}

/// A transcript entry for container execution.
#[derive(Debug, Clone)]
pub struct TranscriptEntry {
    /// Commands that were executed.
    pub commands: Vec<String>,
    /// Output from the commands.
    pub output: String,
}

// ---------------------------------------------------------------------------
// Tag Registry
// ---------------------------------------------------------------------------

/// Registry for tag handlers.
///
/// Handlers are stored by name and can be looked up during tag processing.
/// Multiple handlers can be registered for the same tag; they are sorted by
/// priority (highest first) and the first matching handler is used.
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
        let name = handler.name().to_string();
        let idx = self.handlers.len();
        self.handlers.push(handler);

        self.by_name.entry(name).or_default().push(idx);

        // Sort handlers for this name by priority (descending)
        if let Some(indices) = self.by_name.get_mut(self.handlers[idx].name()) {
            indices.sort_by(|&a, &b| {
                self.handlers[b]
                    .priority()
                    .cmp(&self.handlers[a].priority())
            });
        }
    }

    /// Find the highest-priority handler for a tag name.
    pub fn find(&self, name: &str) -> Option<&dyn TagHandler> {
        self.by_name
            .get(name)
            .and_then(|indices| indices.first())
            .map(|&idx| self.handlers[idx].as_ref())
    }

    /// Get all handlers for a tag name, sorted by priority (highest first).
    pub fn find_all(&self, name: &str) -> Vec<&dyn TagHandler> {
        self.by_name
            .get(name)
            .map(|indices| {
                indices
                    .iter()
                    .map(|&idx| self.handlers[idx].as_ref())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all handlers in a specific phase, sorted by priority.
    pub fn handlers_in_phase(&self, phase: ProcessingPhase) -> Vec<&dyn TagHandler> {
        let mut result: Vec<_> = self
            .handlers
            .iter()
            .filter(|h| h.phase() == phase)
            .map(|h| h.as_ref())
            .collect();
        result.sort_by(|a, b| b.priority().cmp(&a.priority()));
        result
    }

    /// Get all registered handler names.
    pub fn names(&self) -> Vec<&str> {
        self.by_name.keys().map(|s| s.as_str()).collect()
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
        if let HickNode::Text(text) = child {
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
        name: String,
        priority: i32,
    }

    impl TagHandler for TestHandler {
        fn name(&self) -> &str {
            &self.name
        }

        fn process(&self, _tag: &HickTag, _ctx: &ProcessingContext) -> Result<TagResult> {
            Ok(TagResult::Declaration)
        }

        fn priority(&self) -> i32 {
            self.priority
        }
    }

    #[test]
    fn registry_finds_handler() {
        let mut registry = TagRegistry::new();
        registry.register(Box::new(TestHandler {
            name: "test".to_string(),
            priority: 0,
        }));

        assert!(registry.find("test").is_some());
        assert!(registry.find("nonexistent").is_none());
    }

    #[test]
    fn registry_priority_ordering() {
        let mut registry = TagRegistry::new();
        registry.register(Box::new(TestHandler {
            name: "test".to_string(),
            priority: 1,
        }));
        registry.register(Box::new(TestHandler {
            name: "test".to_string(),
            priority: 10,
        }));
        registry.register(Box::new(TestHandler {
            name: "test".to_string(),
            priority: 5,
        }));

        let handlers = registry.find_all("test");
        assert_eq!(handlers.len(), 3);
        // Should be sorted by priority descending: 10, 5, 1
        assert_eq!(handlers[0].priority(), 10);
        assert_eq!(handlers[1].priority(), 5);
        assert_eq!(handlers[2].priority(), 1);
    }

    #[test]
    fn registry_has_handler() {
        let mut registry = TagRegistry::new();
        registry.register(Box::new(TestHandler {
            name: "copy".to_string(),
            priority: 0,
        }));

        assert!(registry.has_handler("copy"));
        assert!(!registry.has_handler("paste"));
    }
}
