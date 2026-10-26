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
        if uri.scheme() != "hick-project" {
            return Err("Not a captured project identity".into());
        }
        let path = uri.path().trim_start_matches('/');
        let (id, file) = path
            .split_once('/')
            .ok_or("Project context needs a file path")?;
        validate_project_path(file)?;
        let context = self
            .contexts
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or("Project context expired; capture the comparison again")?;
        if context.tree.get(file).is_none() {
            return Err(format!("Code absent from selected project: {file}"));
        }
        Ok((id.into(), file.into(), context))
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

### Assemble `crates/hick-lsp/src/project_contexts.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lsp/src/project_contexts.rs"><storycode:paste select="#f-3058901b5b2e-ProjectContexts"/><storycode:paste select="#f-3058901b5b2e-ProjectContext"/><storycode:paste select="#f-3058901b5b2e-impl-ProjectContexts"/><storycode:paste select="#f-3058901b5b2e-impl-ProjectContext"/></storycode:file>

</storycode:doc>
