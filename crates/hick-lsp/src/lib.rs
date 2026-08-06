//! `hick-lsp` is an LSP server for `.hick` files.
//!
//! It parses hick documents, generates virtual files from `hick:file` blocks
//! (resolving `hick:copy`/`hick:paste`), spawns child LSP servers for each
//! output language, and proxies LSP requests by translating positions between
//! `.hick` source coordinates and virtual file coordinates.

pub mod backend;
pub mod child_lsp;
pub mod dispatcher;
pub mod document;
pub mod lang_detect;
pub mod position_map;
pub mod structural;
pub mod virtual_file;

/// The tower-lsp backend implementing the hick LSP multiplexer.
pub use backend::HickBackend;
