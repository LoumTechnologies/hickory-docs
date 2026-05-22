//! Tree-sitter AST ↔ hick element mapping, structural selectors, and source edit computation.
//!
//! `hick-lens` bridges the gap between generated output files and their `.hick`
//! source positions. Given a converged output file and its [`ProvenanceMap`], it
//! uses tree-sitter to parse the output, identifies structural elements (functions,
//! structs, classes, imports), and maps them back to their source `.hick` positions.
//!
//! Key capabilities:
//! - **AST mapping**: Parse output with tree-sitter and correlate AST nodes with provenance spans
//! - **Structural selectors**: Query by `fn name`, `struct Name`, `class Name`, `#id`, `.class`
//! - **Source edit computation**: Given a selector match, compute which `.hick` byte ranges to edit

pub mod annotate;
pub mod edit;
pub mod language;
pub mod mapping;
pub mod paste_slots;
pub mod selector;

pub use edit::{EditConfidence, SourceEdit};
pub use language::Language;
pub use mapping::{AstElement, AstElementKind, AstMapping};
pub use paste_slots::{PasteInsertion, compute_insertions};
pub use selector::{Selector, SelectorMatch};
