<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-lang-wasm/src/lib.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9a9e430ed3f3"></a>
## `crates/hick-lang-wasm/src/lib.rs`

Related contracts: [Literal files use native bytes](../../../../guarantees/embedding/literal-files-use-native-bytes/contract.md), [The Editor Reads A Document With The Parser The Server Uses](../../../../guarantees/language/the-editor-reads-with-the-parser-the-server-uses/contract.md), [Rebuild the program from its Markdown source](../../../../guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md), [Browser-local prediction practice uses the shared gate](../../../../guarantees/verification/browser-predictions-use-the-shared-gate/contract.md).

<a id="f-9a9e430ed3f3-prediction"></a>
### `prediction`

`hick-lang` for the editor.
The web app used to carry a parser of its own for drawing a document's
structure — one regular expression — and it disagreed with `hick-lang`
wherever the grammar is more than that expression: a rebound prefix, a
verbatim element, a document about hick's own syntax. This crate is the
whole fix: the same parser, compiled to WebAssembly, answering the same
question with the same byte offsets.
The surface is one function returning JSON rather than a bound object
graph, so the crate depends on nothing but `wasm-bindgen` and stays a
thin door onto `hick_lang::structure`. Nothing here executes anything:
this is the parser, not a runtime.

<storycode:copy id="f-9a9e430ed3f3-prediction">
//! `hick-lang` for the editor.
//!
//! The web app used to carry a parser of its own for drawing a document's
//! structure — one regular expression — and it disagreed with `hick-lang`
//! wherever the grammar is more than that expression: a rebound prefix, a
//! verbatim element, a document about hick's own syntax. This crate is the
//! whole fix: the same parser, compiled to WebAssembly, answering the same
//! question with the same byte offsets.
//!
//! The surface is one function returning JSON rather than a bound object
//! graph, so the crate depends on nothing but `wasm-bindgen` and stays a
//! thin door onto `hick_lang::structure`. Nothing here executes anything:
//! this is the parser, not a runtime.

use wasm_bindgen::prelude::*;
mod prediction;
</storycode:copy>
<a id="f-9a9e430ed3f3-project"></a>
### `project`

<storycode:copy id="f-9a9e430ed3f3-project">
mod project;
</storycode:copy>
<a id="f-9a9e430ed3f3-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-9a9e430ed3f3-use_declaration">
pub use prediction::browser_prediction;
</storycode:copy>
<a id="f-9a9e430ed3f3-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-9a9e430ed3f3-use_declaration-2">
pub use project::{
    latest_debug_project, project_position, project_transition, regenerate_project_patch,
    reverse_project_edit,
};

</storycode:copy>
<a id="f-9a9e430ed3f3-structure"></a>
### `structure`

The structure of a document — `hick_lang::Structure` as JSON.
Offsets are byte offsets into `source`'s UTF-8; the caller converts them
to whatever its strings index by.

<storycode:copy id="f-9a9e430ed3f3-structure">
/// The structure of a document — `hick_lang::Structure` as JSON.
///
/// Offsets are byte offsets into `source`'s UTF-8; the caller converts them
/// to whatever its strings index by.
#[wasm_bindgen]
pub fn structure(source: &str) -> String {
    serde_json::to_string(&hick_lang::structure(source))
        .expect("a Structure holds only strings, numbers and booleans")
}

</storycode:copy>
<a id="f-9a9e430ed3f3-literal_files"></a>
### `literal_files`

Versioned non-executing materialization of literal files, including lineage.
Missing semantics are a diagnostic, never a successful partial program.

<storycode:copy id="f-9a9e430ed3f3-literal_files">
/// Versioned non-executing materialization of literal files, including lineage.
/// Missing semantics are a diagnostic, never a successful partial program.
#[wasm_bindgen]
pub fn literal_files(source: &str) -> String {
    match hick_lang::literal_files(source) {
        Ok(files) => serde_json::json!({ "version": 1, "files": files, "error": null }).to_string(),
        Err(error) => serde_json::json!({ "version": 1, "files": [], "error": error }).to_string(),
    }
}

</storycode:copy>
<a id="f-9a9e430ed3f3-evaluate_project"></a>
### `evaluate_project`

Pure complete-project evaluation. Input/output entries are byte arrays,
preserving binary assets and metadata; no browser filesystem is consulted.

<storycode:copy id="f-9a9e430ed3f3-evaluate_project">
/// Pure complete-project evaluation. Input/output entries are byte arrays,
/// preserving binary assets and metadata; no browser filesystem is consulted.
#[wasm_bindgen]
pub fn evaluate_project(entry: &str, sources_json: &str, input_json: &str) -> String {
    let result = (|| -> Result<serde_json::Value, String> {
        let sources =
            serde_json::from_str(sources_json).map_err(|e| format!("source closure: {e}"))?;
        let tree = project::read_tree(input_json)?;
        let project = hick_lang::evaluate_project(entry, &sources, &tree)?;
        let serialize = project::serialize;
        let states: serde_json::Map<String, serde_json::Value> = project
            .local_states
            .iter()
            .map(|(id, (old, new))| {
                (
                    id.clone(),
                    serde_json::json!({"old":serialize(old),"new":serialize(new)}),
                )
            })
            .collect();
        let scopes: serde_json::Map<String, serde_json::Value> = project
            .scopes
            .iter()
            .map(|(id, (old, new))| {
                (
                    id.clone(),
                    serde_json::json!({"old":serialize(old),"new":serialize(new)}),
                )
            })
            .collect();
        Ok(
            serde_json::json!({"scopes":scopes,"old":serialize(&project.old),"new":serialize(&project.new),"inventory":project.inventory,"local_states":states,"versions":project.versions.iter().map(serialize).collect::<Vec<_>>()}),
        )
    })();
    match result {
        Ok(project) => serde_json::json!({"version":1,"project":project,"error":null}).to_string(),
        Err(error) => serde_json::json!({"version":1,"project":null,"error":error}).to_string(),
    }
}
</storycode:copy>

### Assemble `crates/hick-lang-wasm/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lang-wasm/src/lib.rs"><storycode:paste select="#f-9a9e430ed3f3-prediction"/><storycode:paste select="#f-9a9e430ed3f3-project"/><storycode:paste select="#f-9a9e430ed3f3-use_declaration"/><storycode:paste select="#f-9a9e430ed3f3-use_declaration-2"/><storycode:paste select="#f-9a9e430ed3f3-structure"/><storycode:paste select="#f-9a9e430ed3f3-literal_files"/><storycode:paste select="#f-9a9e430ed3f3-evaluate_project"/></storycode:file>

</storycode:doc>
