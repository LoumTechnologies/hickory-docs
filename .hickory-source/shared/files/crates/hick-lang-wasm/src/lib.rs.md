<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lang-wasm/src/lib.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Literal files use native bytes](../../../../../guarantees/embedding/literal-files-use-native-bytes/contract.md)
- [The Editor Reads A Document With The Parser The Server Uses](../../../../../guarantees/language/the-editor-reads-with-the-parser-the-server-uses/contract.md)
- [Rebuild the program from its Markdown source](../../../../../guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md)
- [Browser-local prediction practice uses the shared gate](../../../../../guarantees/verification/browser-predictions-use-the-shared-gate/contract.md)

Captured SHA-256: `969265ce60c611813a0302aee809e07d52beaea305c8501331b96fc3fe396326`.

<book:entry path="crates/hick-lang-wasm/src/lib.rs">
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
mod project;
pub use prediction::browser_prediction;
pub use project::{
    latest_debug_project, project_position, project_transition, regenerate_project_patch,
    reverse_project_edit,
};

/// The structure of a document — `hick_lang::Structure` as JSON.
///
/// Offsets are byte offsets into `source`'s UTF-8; the caller converts them
/// to whatever its strings index by.
#[wasm_bindgen]
pub fn structure(source: &str) -> String {
    serde_json::to_string(&hick_lang::structure(source))
        .expect("a Structure holds only strings, numbers and booleans")
}

/// Versioned non-executing materialization of literal files, including lineage.
/// Missing semantics are a diagnostic, never a successful partial program.
#[wasm_bindgen]
pub fn literal_files(source: &str) -> String {
    match hick_lang::literal_files(source) {
        Ok(files) => serde_json::json!({ "version": 1, "files": files, "error": null }).to_string(),
        Err(error) => serde_json::json!({ "version": 1, "files": [], "error": error }).to_string(),
    }
}

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
</book:entry>

</book:doc>
