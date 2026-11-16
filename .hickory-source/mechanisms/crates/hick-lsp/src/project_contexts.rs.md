<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-lsp/src/project_contexts.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-3058901b5b2e"></a>
## `crates/hick-lsp/src/project_contexts.rs`

Related contracts: [Keep language tools and debug launches tied to their captured project](../../../../guarantees/literate-projects/tools-select-complete-isolated-endpoints/contract.md).

<a id="f-3058901b5b2e-ProjectContexts"></a>
### `ProjectContexts`

Immutable comparison projects. Registration is memory-only; native tools
explicitly materialize a complete tree under its own private identity.

<storycode:copy id="f-3058901b5b2e-ProjectContexts">
//! Immutable comparison projects. Registration is memory-only; native tools
//! explicitly materialize a complete tree under its own private identity.
use crate::project::{MaterializedProject, ProjectEndpoint};
use hick_lang::{EvaluatedProject, ProjectTree, validate_project_path};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tower_lsp::lsp_types::Url;

#[derive(Default)]
pub struct ProjectContexts {
    contexts: Mutex<BTreeMap<String, Arc<ProjectContext>>>,
}
</storycode:copy>
<a id="f-3058901b5b2e-ProjectContext"></a>
### `ProjectContext`

<storycode:copy id="f-3058901b5b2e-ProjectContext">
pub struct ProjectContext {
    pub tree: ProjectTree,
    native: Mutex<Option<MaterializedProject>>,
}
</storycode:copy>
<a id="f-3058901b5b2e-impl-ProjectContexts"></a>
### `impl-ProjectContexts`

<storycode:copy id="f-3058901b5b2e-impl-ProjectContexts">
impl ProjectContexts {
    pub fn register(&self, id: &str, tree: ProjectTree) -> Result<String, String> {
        if id.len() != 16 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("Project context requires a captured identity".into());
        }
        if tree.entries().is_empty() {
            return Err("Selected project is empty; there is no executable Old input".into());
        }
        let mut contexts = self.contexts.lock().unwrap();
        if contexts.contains_key(id) {
            return Err("Project context identity already exists".into());
        }
        contexts.insert(
            id.into(),
            Arc::new(ProjectContext {
                tree,
                native: Mutex::new(None),
            }),
        );
        Ok(format!("hick-project:///{id}/"))
    }
    pub fn get(&self, uri: &Url) -> Result<(String, String, Arc<ProjectContext>), String> {
        let (id, file) = Self::source_address(uri)?;
        let context = self
            .contexts
            .lock()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or("Project context expired; capture the comparison again")?;
        if context.tree.get(&file).is_none() {
            return Err(format!("Code absent from selected project: {file}"));
        }
        Ok((id, file, context))
    }
    pub fn remove(&self, id: &str) {
        self.contexts.lock().unwrap().remove(id);
    }
}
</storycode:copy>
<a id="f-3058901b5b2e-impl-ProjectContext"></a>
### `impl-ProjectContext`

<storycode:copy id="f-3058901b5b2e-impl-ProjectContext">
impl ProjectContext {
    pub fn materialize(&self) -> Result<PathBuf, String> {
        let mut native = self.native.lock().unwrap();
        if native.is_none() {
            let project = EvaluatedProject {
                old: ProjectTree::default(),
                new: self.tree.clone(),
                inventory: Default::default(),
                local_states: Default::default(),
                versions: vec![],
                file_paths: Default::default(),
                scopes: Default::default(),
                recordings: vec![],
            };
            *native = Some(crate::project::materialize(&project, ProjectEndpoint::New)?);
        }
        Ok(native.as_ref().unwrap().root().to_path_buf())
    }
    pub fn is_materialized(&self) -> bool {
        self.native.lock().unwrap().is_some()
    }
}
</storycode:copy>


## Preserve captured source addresses across URI and tree boundaries

A literal percent sign or hash is part of a portable output filename, not a URI
escape or fragment. Encode from tree keys and decode once before lookup. Refuse
encoded separators, foreign authorities and query/fragment addresses rather than
borrowing another captured file. The same helper serves language and debugger
providers so their source hashes refer to the same bytes.

<storycode:copy id="captured-source-uri-boundary">
impl ProjectContexts {
    /// Encode a physical project spelling once, retaining path separators.
    pub fn source_uri(id: &str, file: &str) -> Result<String, String> {
        if id.len() != 16 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("Project context requires a captured identity".into());
        }
        validate_project_path(file)?;
        let mut uri = format!("hick-project:///{id}/");
        for byte in file.bytes() {
            if byte.is_ascii_alphanumeric() || b"/-_.!~*'()".contains(&byte) {
                uri.push(byte as char);
            } else {
                uri.push_str(&format!("%{byte:02X}"));
            }
        }
        Ok(uri)
    }
    pub fn decode_source_uri(raw: &str) -> Result<(String, String), String> {
        let uri = Url::parse(raw).map_err(|error| error.to_string())?;
        Self::source_address(&uri)
    }
    /// Decode once; a URI's escaped path is not a ProjectTree key.
    pub fn source_address(uri: &Url) -> Result<(String, String), String> {
        if uri.scheme() != "hick-project"
            || uri.host_str().is_some()
            || uri.query().is_some()
            || uri.fragment().is_some()
        {
            return Err(
                "Use an exact captured source URI without authority, query or fragment".into(),
            );
        }
        let path = uri
            .path()
            .strip_prefix('/')
            .ok_or("Captured source needs a path")?;
        let (id, encoded) = path
            .split_once('/')
            .ok_or("Project context needs a file path")?;
        let mut bytes = Vec::with_capacity(encoded.len());
        let mut input = encoded.as_bytes().iter().copied();
        while let Some(byte) = input.next() {
            if byte == b'%' {
                let high = input.next().and_then(|b| (b as char).to_digit(16));
                let low = input.next().and_then(|b| (b as char).to_digit(16));
                let decoded = high
                    .zip(low)
                    .map(|(a, b)| (a * 16 + b) as u8)
                    .ok_or("Malformed captured source escape")?;
                if decoded == b'/' || decoded == b'\\' {
                    return Err("Encoded source separators are ambiguous".into());
                }
                bytes.push(decoded);
            } else {
                bytes.push(byte);
            }
        }
        let file = String::from_utf8(bytes).map_err(|_| "Captured source is not UTF-8")?;
        Self::source_uri(id, &file)?;
        Ok((id.into(), file))
    }
}
</storycode:copy>

## Check distinct literal identities and ambiguous input refusal

Round-trip two percent-looking names through actual context lookup and check
that neither resolves to the other's bytes. Reject malformed and structurally
ambiguous addresses before any materialization or provider launch.

<storycode:copy id="captured-source-uri-tests">
#[cfg(test)]
mod captured_address_tests {
    use super::*;
    #[test]
    fn literal_captured_paths_round_trip_without_borrowing_another_entry() {
        let contexts = ProjectContexts::default();
        let id = "0123456789abcdef";
        let mut tree = ProjectTree::default();
        for name in ["literal %2F #.py", "literal %252F #.py", "folder/name.py"] {
            tree.insert(
                name,
                hick_lang::ProjectEntry::file(name.as_bytes().to_vec()),
            )
            .unwrap();
        }
        contexts.register(id, tree).unwrap();
        for name in ["literal %2F #.py", "literal %252F #.py", "folder/name.py"] {
            let uri = Url::parse(&ProjectContexts::source_uri(id, name).unwrap()).unwrap();
            let (_, observed, context) = contexts.get(&uri).unwrap();
            assert_eq!(observed, name);
            assert_eq!(
                context.tree.get(&observed).unwrap().bytes.as_ref(),
                name.as_bytes()
            );
        }
    }
    #[test]
    fn malformed_or_ambiguous_captured_addresses_refuse() {
        for raw in [
            "hick-project://other/0123456789abcdef/a.py",
            "hick-project:///0123456789abcdef/a.py#fragment",
            "hick-project:///0123456789abcdef/a.py?query",
            "hick-project:///0123456789abcdef/a%2Fb.py",
            "hick-project:///0123456789abcdef/a%5Cb.py",
            "hick-project:///0123456789abcdef/a%",
            "hick-project:///bad/a.py",
        ] {
            let uri = Url::parse(raw).unwrap();
            assert!(ProjectContexts::source_address(&uri).is_err(), "{raw}");
        }
    }
}
</storycode:copy>

### Assemble `crates/hick-lsp/src/project_contexts.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lsp/src/project_contexts.rs"><storycode:paste select="#f-3058901b5b2e-ProjectContexts"/><storycode:paste select="#f-3058901b5b2e-ProjectContext"/><storycode:paste select="#f-3058901b5b2e-impl-ProjectContexts"/><storycode:paste select="#f-3058901b5b2e-impl-ProjectContext"/><storycode:paste select="#captured-source-uri-boundary"/><storycode:paste select="#captured-source-uri-tests"/></storycode:file>

</storycode:doc>
