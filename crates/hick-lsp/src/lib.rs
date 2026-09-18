//! `hick-lsp` is an LSP server for `.hick` files.
//!
//! It parses hick documents, generates virtual files from `hick:file` blocks
//! (resolving `hick:copy`/`hick:paste`), spawns child LSP servers for each
//! output language, and proxies LSP requests by translating positions between
//! `.hick` source coordinates and virtual file coordinates.

pub mod backend;
pub mod child_lsp;
pub mod discovery;
pub mod dispatcher;
pub mod document;
pub mod element_lint;
pub mod lang_detect;
pub mod position_map;
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
