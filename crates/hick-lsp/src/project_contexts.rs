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
pub struct ProjectContext {
    pub tree: ProjectTree,
    pub recorded_fence: Option<String>,
    native: Mutex<Option<MaterializedProject>>,
}
impl ProjectContexts {
    pub fn register(&self, id: &str, tree: ProjectTree) -> Result<String, String> {
        self.insert(id, tree, None)
    }
    pub fn register_recorded_fence(&self, id: &str, tree: ProjectTree, program: String) -> Result<String, String> {
        let message = ProjectContext::RECORDED_MESSAGE;
        if program == message || tree.get(message).is_none() || tree.get(&program).is_none() {
            return Err("Recorded fence needs its source message and selected program".into());
        }
        self.insert(id, tree, Some(program))?;
        Self::source_uri(id, message)
    }
    fn insert(&self, id: &str, tree: ProjectTree, recorded_fence: Option<String>) -> Result<String, String> {
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
        if recorded_fence.is_some() && contexts.values().filter(|context| context.recorded_fence.is_some()).count() >= 64 {
            return Err("Too many recorded fences are open. Close an earlier fence and retry.".into());
        }
        contexts.insert(
            id.into(),
            Arc::new(ProjectContext {
                tree,
                recorded_fence,
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
impl ProjectContext {
    pub const RECORDED_MESSAGE: &str = "message.md";

    pub fn materialize(&self) -> Result<PathBuf, String> {
        let mut native = self.native.lock().unwrap();
        if native.is_none() {
            let project = EvaluatedProject {
                patch_context_lines: Vec::new(),
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
