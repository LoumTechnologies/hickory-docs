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
