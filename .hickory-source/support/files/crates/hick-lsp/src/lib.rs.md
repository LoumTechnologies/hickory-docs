<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lsp/src/lib.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `f0ac35730f20f2a226bf1aaa401e12085624a8c69bc28973174ab23c135a3d3f`.

<book:entry path="crates/hick-lsp/src/lib.rs">
//! `hick-lsp` is an LSP server for `.md` files.
//!
//! It parses hick documents, generates virtual files from `hick:file` blocks
//! (resolving `hick:copy`/`hick:paste`), spawns child LSP servers for each
//! output language, and proxies LSP requests by translating positions between
//! `.md` source coordinates and virtual file coordinates.

pub mod backend;
pub mod child_lsp;
pub mod discovery;
pub mod dispatcher;
pub mod document;
pub mod element_lint;
pub mod lang_detect;
mod location_map;
mod project_contexts;
pub use project_contexts::{ProjectContext, ProjectContexts};
pub mod position_map;
pub mod project;
pub mod project_environment;
mod project_positions;
pub mod semantic;
pub mod server_config;
pub mod session_lint;
pub mod staging;
pub mod structural;
pub mod virtual_file;

/// The tower-lsp backend implementing the hick LSP multiplexer.
pub use backend::HickBackend;
/// Where a document's code is staged for the child language servers, and how
/// a path under there is named back to the output path the document gave it.
pub use staging::{STAGING_PREFIX, StagingArea, StagingError, staged_output_path};

pub use project::{
    MaterializedProject, ProjectEndpoint, debug_selection, endpoint, latest_file, materialize,
    to_endpoint, to_source, utf16_position, write_tree,
};
</book:entry>

</book:doc>
