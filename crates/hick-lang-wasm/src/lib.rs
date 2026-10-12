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
        let input: Vec<serde_json::Value> =
            serde_json::from_str(input_json).map_err(|e| format!("input project: {e}"))?;
        let mut tree = hick_lang::ProjectTree::default();
        for item in input {
            let path = item["path"].as_str().ok_or("entry requires path")?;
            if tree.get(path).is_some() {
                return Err(format!("duplicate input path: {path}"));
            }
            let bytes: Vec<u8> =
                serde_json::from_value(item["bytes"].clone()).map_err(|e| e.to_string())?;
            let object = item.as_object().ok_or("project entry must be an object")?;
            if object.keys().any(|key| {
                !["path", "bytes", "kind", "executable", "origins"].contains(&key.as_str())
            }) {
                return Err(format!("unsupported input entry metadata for {path}"));
            }
            let kind_name = match object.get("kind") {
                None => "file",
                Some(value) => value.as_str().ok_or("input kind must be a string")?,
            };
            let kind = match kind_name {
                "file" => hick_lang::EntryKind::File,
                "symlink" => hick_lang::EntryKind::Symlink,
                other => return Err(format!("unsupported input kind: {other}")),
            };
            let executable = match object.get("executable") {
                None => false,
                Some(value) => value
                    .as_bool()
                    .ok_or("input executable metadata must be a boolean")?,
            };
            let origins = match object.get("origins") {
                None => vec![],
                Some(value) => serde_json::from_value(value.clone())
                    .map_err(|e| format!("input origins: {e}"))?,
            };
            tree.insert(
                path,
                hick_lang::ProjectEntry {
                    bytes: bytes.into(),
                    kind,
                    executable,
                    origins,
                },
            )?;
        }
        let project = hick_lang::evaluate_project(entry, &sources, &tree)?;
        let serialize = |tree: &hick_lang::ProjectTree| {
            tree.entries().iter().map(|(path,e)|serde_json::json!({"path":path,"bytes":e.bytes.as_ref(),"kind":match e.kind {hick_lang::EntryKind::File=>"file",hick_lang::EntryKind::Symlink=>"symlink"},"executable":e.executable,"origins":e.origins})).collect::<Vec<_>>()
        };
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
        Ok(
            serde_json::json!({"old":serialize(&project.old),"new":serialize(&project.new),"inventory":project.inventory,"local_states":states}),
        )
    })();
    match result {
        Ok(project) => serde_json::json!({"version":1,"project":project,"error":null}).to_string(),
        Err(error) => serde_json::json!({"version":1,"project":null,"error":error}).to_string(),
    }
}
