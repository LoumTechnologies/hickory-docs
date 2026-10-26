<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-lsp/src/lib.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-e8bff405197e"></a>
## `crates/hick-lsp/src/lib.rs`

<a id="f-e8bff405197e-backend"></a>
### `backend`

`hick-lsp` is an LSP server for `.md` files.
It parses hick documents, generates virtual files from `hick:file` blocks
(resolving `hick:copy`/`hick:paste`), spawns child LSP servers for each
output language, and proxies LSP requests by translating positions between
`.md` source coordinates and virtual file coordinates.

<storycode:copy id="f-e8bff405197e-backend">
//! `hick-lsp` is an LSP server for `.md` files.
//!
//! It parses hick documents, generates virtual files from `hick:file` blocks
//! (resolving `hick:copy`/`hick:paste`), spawns child LSP servers for each
//! output language, and proxies LSP requests by translating positions between
//! `.md` source coordinates and virtual file coordinates.

pub mod backend;
</storycode:copy>
<a id="f-e8bff405197e-child_lsp"></a>
### `child_lsp`

<storycode:copy id="f-e8bff405197e-child_lsp">
pub mod child_lsp;
</storycode:copy>
<a id="f-e8bff405197e-discovery"></a>
### `discovery`

<storycode:copy id="f-e8bff405197e-discovery">
pub mod discovery;
</storycode:copy>
<a id="f-e8bff405197e-dispatcher"></a>
### `dispatcher`

<storycode:copy id="f-e8bff405197e-dispatcher">
pub mod dispatcher;
</storycode:copy>
<a id="f-e8bff405197e-document"></a>
### `document`

<storycode:copy id="f-e8bff405197e-document">
pub mod document;
</storycode:copy>
<a id="f-e8bff405197e-element_lint"></a>
### `element_lint`

<storycode:copy id="f-e8bff405197e-element_lint">
pub mod element_lint;
</storycode:copy>
<a id="f-e8bff405197e-lang_detect"></a>
### `lang_detect`

<storycode:copy id="f-e8bff405197e-lang_detect">
pub mod lang_detect;
</storycode:copy>
<a id="f-e8bff405197e-location_map"></a>
### `location_map`

<storycode:copy id="f-e8bff405197e-location_map">
mod location_map;
</storycode:copy>
<a id="f-e8bff405197e-project_contexts"></a>
### `project_contexts`

<storycode:copy id="f-e8bff405197e-project_contexts">
mod project_contexts;
</storycode:copy>
<a id="f-e8bff405197e-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-e8bff405197e-use_declaration">
pub use project_contexts::{ProjectContext, ProjectContexts};
</storycode:copy>
<a id="f-e8bff405197e-position_map"></a>
### `position_map`

<storycode:copy id="f-e8bff405197e-position_map">
pub mod position_map;
</storycode:copy>
<a id="f-e8bff405197e-project"></a>
### `project`

<storycode:copy id="f-e8bff405197e-project">
pub mod project;
</storycode:copy>
<a id="f-e8bff405197e-project_environment"></a>
### `project_environment`

<storycode:copy id="f-e8bff405197e-project_environment">
pub mod project_environment;
</storycode:copy>
<a id="f-e8bff405197e-project_positions"></a>
### `project_positions`

<storycode:copy id="f-e8bff405197e-project_positions">
mod project_positions;
</storycode:copy>
<a id="f-e8bff405197e-semantic"></a>
### `semantic`

<storycode:copy id="f-e8bff405197e-semantic">
pub mod semantic;
</storycode:copy>
<a id="f-e8bff405197e-server_config"></a>
### `server_config`

<storycode:copy id="f-e8bff405197e-server_config">
pub mod server_config;
</storycode:copy>
<a id="f-e8bff405197e-session_lint"></a>
### `session_lint`

<storycode:copy id="f-e8bff405197e-session_lint">
pub mod session_lint;
</storycode:copy>
<a id="f-e8bff405197e-staging"></a>
### `staging`

<storycode:copy id="f-e8bff405197e-staging">
pub mod staging;
</storycode:copy>
<a id="f-e8bff405197e-structural"></a>
### `structural`

<storycode:copy id="f-e8bff405197e-structural">
pub mod structural;
</storycode:copy>
<a id="f-e8bff405197e-virtual_file"></a>
### `virtual_file`

<storycode:copy id="f-e8bff405197e-virtual_file">
pub mod virtual_file;

</storycode:copy>
<a id="f-e8bff405197e-use_declaration-2"></a>
### `use_declaration`

The tower-lsp backend implementing the hick LSP multiplexer.

<storycode:copy id="f-e8bff405197e-use_declaration-2">
/// The tower-lsp backend implementing the hick LSP multiplexer.
pub use backend::HickBackend;
</storycode:copy>
<a id="f-e8bff405197e-use_declaration-3"></a>
### `use_declaration`

Where a document's code is staged for the child language servers, and how
a path under there is named back to the output path the document gave it.

<storycode:copy id="f-e8bff405197e-use_declaration-3">
/// Where a document's code is staged for the child language servers, and how
/// a path under there is named back to the output path the document gave it.
pub use staging::{STAGING_PREFIX, StagingArea, StagingError, staged_output_path};

</storycode:copy>
<a id="f-e8bff405197e-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-e8bff405197e-use_declaration-4">
pub use project::{
    MaterializedProject, ProjectEndpoint, debug_selection, endpoint, latest_file, materialize,
    to_endpoint, to_source, utf16_position, write_tree,
};
</storycode:copy>

### Assemble `crates/hick-lsp/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lsp/src/lib.rs"><storycode:paste select="#f-e8bff405197e-backend"/><storycode:paste select="#f-e8bff405197e-child_lsp"/><storycode:paste select="#f-e8bff405197e-discovery"/><storycode:paste select="#f-e8bff405197e-dispatcher"/><storycode:paste select="#f-e8bff405197e-document"/><storycode:paste select="#f-e8bff405197e-element_lint"/><storycode:paste select="#f-e8bff405197e-lang_detect"/><storycode:paste select="#f-e8bff405197e-location_map"/><storycode:paste select="#f-e8bff405197e-project_contexts"/><storycode:paste select="#f-e8bff405197e-use_declaration"/><storycode:paste select="#f-e8bff405197e-position_map"/><storycode:paste select="#f-e8bff405197e-project"/><storycode:paste select="#f-e8bff405197e-project_environment"/><storycode:paste select="#f-e8bff405197e-project_positions"/><storycode:paste select="#f-e8bff405197e-semantic"/><storycode:paste select="#f-e8bff405197e-server_config"/><storycode:paste select="#f-e8bff405197e-session_lint"/><storycode:paste select="#f-e8bff405197e-staging"/><storycode:paste select="#f-e8bff405197e-structural"/><storycode:paste select="#f-e8bff405197e-virtual_file"/><storycode:paste select="#f-e8bff405197e-use_declaration-2"/><storycode:paste select="#f-e8bff405197e-use_declaration-3"/><storycode:paste select="#f-e8bff405197e-use_declaration-4"/></storycode:file>

</storycode:doc>
