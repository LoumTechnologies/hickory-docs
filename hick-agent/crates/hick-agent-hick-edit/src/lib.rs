//! Hick-XML-aware agent extension crate.
//!
//! Consolidates all agent behaviour specific to the Hickory XML pipeline
//! format: edit format parsing, pipeline watching, reverse provenance, file
//! I/O, and high-level composition helpers.
//!
//! Extends [`hick_agent_sdk`] without polluting the generic SDK with
//! Hickory-specific knowledge.
//!
//! # Key types
//!
//! | Type | Purpose |
//! |------|---------|
//! | [`HickEditAgent`] | Composable hick-edit behaviour layered on [`AgentHistory`] |
//! | [`HickEditFormat`] | Trait for hick XML edit operations |
//! | [`HickEditService`] | Concrete edit service backed by hick and hick-lens |
//! | [`PipelineWatcher`] | Watches `.hick` files and transparently reruns the pipeline |
//! | [`PasteSlotRegistry`] | Index of named paste slots in the current pipeline |

pub mod edit_format;
pub mod file_io;
pub mod hick_edit_agent;
pub mod paste_slot_inserter;
pub mod paste_slot_registry;
pub mod pipeline_registry;
pub mod pipeline_watcher;
pub mod reverse_provenance;

/// System prompt fragment describing Hickory XML behaviour and generated-file
/// policy. Call this from any agent binary that depends on this crate and
/// concatenate the result into the agent's system prompt.
pub fn system_prompt_fragment() -> &'static str {
    "## Hickory XML: key behaviours for pipeline editing

Hickory XML is NOT standard XML. The parser only recognises elements that carry
the `hick:` namespace prefix (e.g. `<hick:doc>`, `<hick:file>`, `<hick:cut>`).
Everything else is passed through **literally** — bare `<`, `>`, and `&` in
content are never parsed, escaped, or modified.

**Never escape source code in .hick files.** Write Rust generics, C++
templates, shell `&&`, HTML, etc. with bare characters:
  pub fn foo<T>() -> Result<T>          ← correct
  pub fn foo&lt;T&gt;() -> Result&lt;T&gt;   ← wrong — entity refs appear verbatim in output

If an output file contains `&lt;`, `&gt;`, or `&amp;` as literal text, the
`.hick` source has pre-escaped content. Fix the `.hick` source — do not edit
the output file.

**Output files are generated — never edit them directly.** The next pipeline
run overwrites any direct edits. Always check before writing:
  hick.is_pipeline_owned(path)     → True = edit the .hick source instead
  hick.provenance(path, start, end) → find which .hick source owns these lines"
}

// ---------------------------------------------------------------------------
// Edit format
// ---------------------------------------------------------------------------

/// Trait for Hick XML edit operations: run pipelines, query structure, compute and apply edits.
pub use edit_format::HickEditFormat;
/// Concrete implementation of [`HickEditFormat`] backed by hick and hick-lens.
pub use edit_format::HickEditService;
/// Output from a pipeline run, including generated files, provenance maps, and AST mappings.
pub use edit_format::PipelineOutput;
/// A single structural query match with element metadata and provenance origins.
pub use edit_format::QueryMatch;
/// A computed source edit (file, byte range, replacement content, confidence).
pub use edit_format::SourceEdit;

// ---------------------------------------------------------------------------
// Pipeline watcher
// ---------------------------------------------------------------------------

/// Watches `.hick` files, transparently reruns the pipeline, and applies
/// reverse provenance when output files are edited directly.
pub use pipeline_watcher::PipelineWatcher;
/// Metadata written to `.hick/pipeline.state` after each successful run.
pub use pipeline_watcher::PipelineState;

// ---------------------------------------------------------------------------
// Paste slot registry
// ---------------------------------------------------------------------------

/// Indexes named `<hick:paste>` slots so the agent can discover composition
/// points without scanning every source file on each request.
pub use paste_slot_registry::PasteSlotRegistry;
/// Insert `<hick:paste name="…"/>` slots into `hick:file` content at structural positions.
pub use paste_slot_inserter::insert_paste_slots;

// ---------------------------------------------------------------------------
// Hick edit agent
// ---------------------------------------------------------------------------

/// Composable hick-XML edit behaviour layered on top of [`hick_agent_sdk::AgentHistory`].
pub use hick_edit_agent::HickEditAgent;

// ---------------------------------------------------------------------------
// Pipeline registry
// ---------------------------------------------------------------------------

/// Maps pipeline output file paths to their owning `.hick` sources and paste slots.
pub use pipeline_registry::PipelineRegistry;
/// Ownership record for one pipeline-managed output file.
pub use pipeline_registry::FileOwnership;
