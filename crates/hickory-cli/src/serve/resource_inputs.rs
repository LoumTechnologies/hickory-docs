//! Resource inputs are saved observations plus explicit draft overlays, never an implicit save.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
    resource_changes::{self, SourceState},
};
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[cfg(test)]
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};
/// Root identity is an independent capability; ordinary text opening never waits for it.
pub struct RootIdentity {
    bound: std::sync::OnceLock<Result<(same_file::Handle, PathBuf, String), String>>,
    ready: tokio::sync::Notify,
}
impl RootIdentity {
    fn binding(root: &Path) -> Result<(same_file::Handle, PathBuf, String), String> {
        let path = root.canonicalize().map_err(|error| error.to_string())?;
        let handle = same_file::Handle::from_path(&path).map_err(|error| error.to_string())?;
        let mut nonce = [0u8; 16];
        getrandom::fill(&mut nonce).map_err(|error| error.to_string())?;
        Ok((handle, path, hex::encode(nonce)))
    }
    pub fn bind(root: &Path) -> Self {
        Self {
            bound: std::sync::OnceLock::from(Self::binding(root)),
            ready: tokio::sync::Notify::new(),
        }
    }
    pub fn start(root: PathBuf) -> Arc<Self> {
        let identity = Arc::new(Self {
            bound: std::sync::OnceLock::new(),
            ready: tokio::sync::Notify::new(),
        });
        let pending = identity.clone();
        tokio::task::spawn_blocking(move || {
            let result = Self::binding(&root);
            let _ = pending.bound.set(result);
            pending.ready.notify_waiters();
        });
        identity
    }
    pub async fn ready(&self) -> Result<(), String> {
        loop {
            let notified = self.ready.notified();
            if let Some(result) = self.bound.get() {
                return result.as_ref().map(|_| ()).map_err(Clone::clone);
            }
            notified.await;
        }
    }
    pub fn check(&self, root: &Path) -> Result<&str, String> {
        let bound=self.bound.get().ok_or("Workspace root identity is still loading. Prepare the resource proposal again once it is ready.")?;
        let (original, path, id) = bound.as_ref().map_err(
            |_| "Workspace root identity is unavailable. Reconnect before resource changes.",
        )?;
        let current = root
            .canonicalize()
            .map_err(|_| "Workspace root is unavailable. Reconnect before resource changes.")?;
        let handle = same_file::Handle::from_path(root)
            .map_err(|_| "Workspace root is unavailable. Reconnect before resource changes.")?;
        if &current != path || &handle != original {
            return Err(
                "Workspace root changed. Reconnect and prepare the resource proposal again.".into(),
            );
        }
        Ok(id)
    }
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub text: String,
    pub base_sha256: String,
    pub version: Option<i32>,
    pub writable: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub context: String,
    pub edit: Value,
    #[serde(default)]
    pub drafts: BTreeMap<String, Draft>,
    pub project_entry: Option<String>,
    #[serde(default)]
    pub ownership_sources: BTreeMap<String, String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SavedSource {
    Missing,
    File {
        text: String,
        sha256: String,
        readonly: bool,
        executable: bool,
        mode: Option<u32>,
    },
}
#[derive(Debug, Serialize)]
pub struct Inputs {
    pub version: u32,
    pub context: String,
    pub workspace: String,
    pub ownership: Ownership,
    pub sources: BTreeMap<String, SourceState>,
    pub saved_sources: BTreeMap<String, SavedSource>,
    pub refusals: BTreeMap<String, String>,
    pub source_files_modified: bool,
}
const MAX_SOURCE: usize = 8 * 1024 * 1024;
const MAX_INPUT: usize = 16 * 1024 * 1024;
fn sha(bytes: &[u8]) -> String {
    super::workspace_fs::view::hash(bytes)
}
struct Names {
    entries: BTreeMap<PathBuf, BTreeSet<std::ffi::OsString>>,
    count: usize,
    bytes: usize,
}
impl Names {
    fn exact(&mut self, parent: &Path, name: &std::ffi::OsStr) -> Result<bool, String> {
        if !self.entries.contains_key(parent) {
            let mut names = BTreeSet::new();
            for entry in std::fs::read_dir(parent).map_err(|error| {
                format!("Source directory cannot be read: {error}. Restore access and retry.")
            })? {
                let entry = entry.map_err(|error| {
                    format!("Source directory changed: {error}. Refresh and retry.")
                })?;
                let name = entry.file_name();
                self.count += 1;
                self.bytes += name.as_encoded_bytes().len();
                if self.count > 100000 || self.bytes > 8 * 1024 * 1024 {
                    return Err("Source directory identity budget exceeded. Open a smaller scope and prepare again.".into());
                }
                names.insert(name);
            }
            self.entries.insert(parent.into(), names);
        }
        Ok(self.entries[parent].contains(name))
    }
}
fn confined(root: &Path, path: &str, names: &mut Names) -> Result<PathBuf, String> {
    resource_changes::validate_path(path)?;
    super::workspace_fs::view::writable(path).map_err(|error| error.to_string())?;
    let mut current = root.to_path_buf();
    let parts: Vec<_> = path.split('/').collect();
    for (index, part) in parts.iter().enumerate() {
        let candidate = current.join(part);
        match std::fs::symlink_metadata(&candidate) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(format!(
                        "Source follows a symlink: {path}. Choose the authored workspace file."
                    ));
                }
                if !names.exact(&current, std::ffi::OsStr::new(part))? {
                    return Err(format!(
                        "Source uses an aliased filename spelling: {path}. Reopen its exact listed path."
                    ));
                }
                if index + 1 < parts.len() && !metadata.is_dir() {
                    return Err(format!("Source parent is not a directory: {path}"));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Source cannot be inspected: {path}: {error}. Restore access and retry."
                ));
            }
        }
        current = candidate;
    }
    Ok(current)
}
fn read_saved(path: &Path) -> Result<SavedSource, String> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(SavedSource::Missing);
        }
        Err(error) => {
            return Err(format!(
                "Source cannot be read: {error}. Restore access and retry."
            ));
        }
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("Resource proposals currently require regular source files. A directory, symlink or special file was not acquired.".into());
    }
    if metadata.len() > MAX_SOURCE as u64 {
        return Err("Source exceeds 8 MiB. Narrow the change.".into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .and_then(|file| file.take(MAX_SOURCE as u64 + 1).read_to_end(&mut bytes))
        .map_err(|error| format!("Source could not be read: {error}. Restore access and retry."))?;
    if bytes.len() > MAX_SOURCE {
        return Err("Source grew beyond 8 MiB. Prepare it again.".into());
    }
    let sha256 = sha(&bytes);
    let text = String::from_utf8(bytes).map_err(
        |_| "Resource proposals require UTF-8 source text; binary contents were not acquired.",
    )?;
    #[cfg(unix)]
    let mode = {
        use std::os::unix::fs::PermissionsExt;
        Some(metadata.permissions().mode() & 0o7777)
    };
    #[cfg(not(unix))]
    let mode: Option<u32> = None;
    Ok(SavedSource::File {
        text,
        sha256,
        readonly: metadata.permissions().readonly(),
        executable: mode.is_some_and(|mode| mode & 0o111 != 0),
        mode,
    })
}
#[derive(Debug, Serialize)]
pub struct Ownership {
    pub mode: String,
    pub entry: Option<String>,
    pub saved_sources: BTreeMap<String, String>,
    pub draft_sources: BTreeMap<String, String>,
}
fn ownership(
    root: &Path,
    index: &super::store::DocIndex,
    request: &Request,
) -> Result<(Ownership, BTreeSet<String>), String> {
    let entry = if let Some(entry) = &request.project_entry {
        resource_changes::validate_path(entry)?;
        Some(entry.clone())
    } else {
        match read_saved(&root.join("project.md"))? {
            SavedSource::Missing => None,
            SavedSource::File { text, .. } => {
                let document=hick_lang::parse(&text).map_err(|error|format!("Construction entry cannot be parsed: {error}. Correct it before resource changes."))?;
                (!document.find_tags("project").is_empty()).then(|| "project.md".to_string())
            }
        }
    };
    let Some(entry) = entry else {
        for source in request.ownership_sources.values() {
            let document = hick_lang::parse(source).map_err(|error| {
                format!("Draft ownership cannot be analyzed: {error}. Choose a construction entry.")
            })?;
            if document
                .all_tags()
                .iter()
                .any(|tag| matches!(tag.name.as_str(), "project" | "file" | "entry" | "volume"))
            {
                return Err("A draft contains construction declarations without a selected entry. Choose its construction entry before resource changes.".into());
            }
        }
        let generated = super::api::generated_outputs(root, index);
        let targets = resource_changes::targets(&request.edit)?;
        let protected = targets
            .into_iter()
            .filter(|path| super::api::generated_by(&generated, path).is_some())
            .collect();
        return Ok((
            Ownership {
                mode: "saved claims and filesystem permission metadata; no selected construction"
                    .into(),
                entry: None,
                saved_sources: BTreeMap::new(),
                draft_sources: BTreeMap::new(),
            },
            protected,
        ));
    };
    let source = super::source_tests::saved_source(root, &entry)?;
    let before = super::source_tests::scan_with_sources(
        root,
        &super::source_tests::Request {
            path: entry.clone(),
            source: source.clone(),
            project_entry: Some(entry.clone()),
            sources: BTreeMap::new(),
            capture: false,
        },
        None,
    )?;
    let after = super::source_tests::scan_with_sources(
        root,
        &super::source_tests::Request {
            path: entry.clone(),
            source: request
                .ownership_sources
                .get(&entry)
                .cloned()
                .unwrap_or(source),
            project_entry: Some(entry.clone()),
            sources: request.ownership_sources.clone(),
            capture: false,
        },
        Some(&before.source_texts),
    )?;
    let base = Path::new(&entry).parent().unwrap_or(Path::new(""));
    let mut generated = BTreeSet::new();
    for scan in [&before, &after] {
        let tree=scan.tree.as_ref().ok_or("Construction ownership is unavailable. Choose a valid project entry before resource changes.")?;
        for path in tree.entries().keys() {
            generated.insert(super::source_tests::relative(base, path)?);
        }
    }
    Ok((
        Ownership {
            mode: "compiler outputs from saved and supplied draft construction inputs".into(),
            entry: Some(entry),
            saved_sources: before.source_hashes,
            draft_sources: after.source_hashes,
        },
        generated,
    ))
}
fn capture(
    root: &Path,
    index: &super::store::DocIndex,
    identity: &RootIdentity,
    request: Request,
) -> Result<Inputs, String> {
    if request.context.is_empty() || request.context.len() > 4096 {
        return Err(
            "Capture the current workspace/provider context before acquiring resource inputs."
                .into(),
        );
    }
    if request.ownership_sources.len() > 4096
        || request
            .ownership_sources
            .values()
            .any(|source| source.len() > MAX_SOURCE)
        || request
            .ownership_sources
            .values()
            .map(String::len)
            .sum::<usize>()
            > MAX_INPUT
    {
        return Err(
            "Draft construction context is too large. Narrow the resource proposal.".into(),
        );
    }
    let workspace = identity.check(root)?.to_owned();
    let root = root.canonicalize().map_err(|error| error.to_string())?;
    let paths = resource_changes::targets(&request.edit)?;
    let selected: BTreeSet<_> = paths.iter().cloned().collect();
    if request.drafts.keys().any(|path| !selected.contains(path)) {
        return Err("A draft was supplied outside this proposal’s discovered source set. Acquire the exact target set again.".into());
    }
    let (ownership, generated) = ownership(&root, index, &request)?;
    let mut names = Names {
        entries: BTreeMap::new(),
        count: 0,
        bytes: 0,
    };
    let (mut sources, mut saved) = (BTreeMap::new(), BTreeMap::new());
    let mut refusals = BTreeMap::new();
    let mut size = 0usize;
    for path in paths {
        let full = confined(&root, &path, &mut names)?;
        let observed = read_saved(&full)?;
        let parent = full.parent().ok_or("Source needs a parent directory.")?;
        let parent_writable = match std::fs::metadata(parent) {
            Ok(metadata) => metadata.is_dir() && !metadata.permissions().readonly(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(error) => {
                return Err(format!(
                    "Source parent cannot be inspected: {error}. Restore access and retry."
                ));
            }
        };
        let source = match &observed {
            SavedSource::Missing => {
                if request.drafts.contains_key(&path) {
                    return Err(format!(
                        "Saved source disappeared or a named draft has no saved baseline: {path}. Preserve it and choose a new destination explicitly."
                    ));
                }
                SourceState::Missing {
                    creatable: parent_writable && !generated.contains(&path),
                }
            }
            SavedSource::File {
                text,
                sha256,
                readonly,
                ..
            } => {
                let draft = request.drafts.get(&path);
                if let Some(draft) = draft {
                    if draft.text.len() > MAX_SOURCE || draft.base_sha256 != *sha256 {
                        return Err(format!(
                            "Saved source changed since the editor’s base: {path}. Preserve the draft, compare or reload, and prepare again."
                        ));
                    }
                }
                let text = draft.map_or(text, |draft| &draft.text).clone();
                size = size.saturating_add(text.len());
                let writable = !*readonly
                    && parent_writable
                    && !generated.contains(&path)
                    && draft.is_none_or(|draft| draft.writable);
                let mut nonce = [0u8; 16];
                getrandom::fill(&mut nonce).map_err(|error| {
                    format!("Source identity could not be captured: {error}. Retry the proposal.")
                })?;
                SourceState::File {
                    identity: hex::encode(nonce),
                    text,
                    writable,
                    removable: writable,
                    version: draft.and_then(|draft| draft.version),
                }
            }
        };
        if let SavedSource::File { text, .. } = &observed {
            size = size.saturating_add(text.len());
        }
        if size > MAX_INPUT {
            return Err("Resource input snapshot exceeds 16 MiB. Narrow the proposal.".into());
        }
        let allowed = match &source {
            SourceState::File { writable, .. } => *writable,
            SourceState::Missing { creatable } => *creatable,
        };
        if !allowed {
            let reason = if generated.contains(&path) {
                "Generated output cannot be changed directly. Open Construction reading and edit its authored source."
            } else if !parent_writable {
                "The source or destination folder is missing or read-only. Restore access or choose an existing writable folder."
            } else if matches!(&observed, SavedSource::File { readonly: true, .. }) {
                "The saved source is read-only. Restore access before preparing this change."
            } else {
                "The editor marked this source unavailable for changes. Open its authored source or finish the pending change, then prepare again."
            };
            refusals.insert(path.clone(), reason.to_string());
        }
        sources.insert(path.clone(), source);
        saved.insert(path, observed);
    }
    identity.check(&root)?;
    let result = Inputs {
        version: 1,
        context: request.context,
        workspace,
        ownership,
        sources,
        saved_sources: saved,
        refusals,
        source_files_modified: false,
    };
    if serde_json::to_vec(&result)
        .map_err(|error| error.to_string())?
        .len()
        > MAX_INPUT
    {
        return Err(
            "Resource input response exceeds its interactive JSON budget. Narrow the proposal."
                .into(),
        );
    }
    Ok(result)
}
static INPUT_READS: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(2);
pub async fn inspect(
    State(state): State<LocalState>,
    Json(request): Json<Request>,
) -> ApiResult<Json<Inputs>> {
    state
        .resource_root
        .ready()
        .await
        .map_err(ApiError::unprocessable)?;
    let permit = INPUT_READS.try_acquire().map_err(|_| {
        ApiError::conflict(
            "Resource analysis is busy. Let the current read finish and prepare again.",
        )
    })?;
    let root = state.index.root().to_path_buf();
    let index = state.index.clone();
    let identity = state.resource_root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        capture(&root, &index, &identity, request)
    })
    .await
    .map_err(|error| {
        ApiError::internal(format!(
            "Resource input acquisition failed: {error}. Prepare the proposal again."
        ))
    })?
    .map_err(ApiError::unprocessable)?;
    for (path, saved) in &result.saved_sources {
        if let SavedSource::File { text, .. } = saved {
            let id = state.index.id_for_path(path);
            if let Some(room) = state.rooms.get(&id).await {
                if room.text().await != *text {
                    return Err(ApiError::conflict(format!(
                        "A live document differs from its saved baseline: {path}. Save or reconcile that document before resource changes."
                    )));
                }
            }
        }
    }
    let root = state.index.root().to_path_buf();
    let identity = state.resource_root.clone();
    let expected = result.workspace.clone();
    tokio::task::spawn_blocking(move || {
        let current = identity.check(&root)?;
        if current != expected {
            return Err(
                "Workspace root changed during acquisition. Reconnect and prepare again.".into(),
            );
        }
        Ok::<_, String>(())
    })
    .await
    .map_err(|error| ApiError::internal(error.to_string()))?
    .map_err(ApiError::conflict)?;
    Ok(Json(result))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(edit: Value) -> Request {
        Request {
            context: "fixture".into(),
            edit,
            drafts: BTreeMap::new(),
            project_entry: None,
            ownership_sources: BTreeMap::new(),
        }
    }
    fn capture_at(root: &Path, request: Request) -> Result<Inputs, String> {
        let index = super::super::store::DocIndex::scan(root).unwrap();
        let identity = RootIdentity::bind(root);
        capture(root, &index, &identity, request)
    }
    #[test]
    fn saved_bytes_and_unsaved_drafts_are_separate_and_no_resource_is_published() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        std::fs::write(root.join("a.py"), "saved\n").unwrap();
        let mut request = fixture(
            json!({"documentChanges":[{"kind":"rename","oldUri":"hick:///a.py","newUri":"hick:///b.py"}]}),
        );
        request.drafts.insert(
            "a.py".into(),
            Draft {
                text: "saved\n# draft\n".into(),
                base_sha256: sha(b"saved\n"),
                version: Some(7),
                writable: true,
            },
        );
        let inputs = capture_at(root, request).unwrap();
        assert!(
            matches!(&inputs.sources["a.py"],SourceState::File{text,version:Some(7),..} if text=="saved\n# draft\n")
        );
        assert!(
            matches!(&inputs.saved_sources["a.py"],SavedSource::File{text,..} if text=="saved\n")
        );
        assert_eq!(inputs.saved_sources["b.py"], SavedSource::Missing);
        assert!(matches!(
            inputs.sources["b.py"],
            SourceState::Missing { creatable: true }
        ));
        assert!(!inputs.workspace.is_empty());
        assert_eq!(std::fs::read(root.join("a.py")).unwrap(), b"saved\n");
        assert!(!root.join("b.py").exists());
    }
    #[test]
    fn stale_bases_and_unknown_drafts_refuse_instead_of_overwriting_external_changes() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        std::fs::write(root.join("a.py"), "external").unwrap();
        let edit = json!({"changes":{"hick:///a.py":[]}});
        let mut request = fixture(edit);
        request.drafts.insert(
            "a.py".into(),
            Draft {
                text: "my draft".into(),
                base_sha256: sha(b"old"),
                version: None,
                writable: true,
            },
        );
        assert!(
            capture_at(root, request)
                .unwrap_err()
                .contains("Saved source changed")
        );
        let mut request = fixture(json!({"changes":{"hick:///a.py":[]}}));
        request.drafts.insert(
            "unrelated.py".into(),
            Draft {
                text: "draft".into(),
                base_sha256: sha(b"old"),
                version: None,
                writable: true,
            },
        );
        assert!(
            capture_at(root, request)
                .unwrap_err()
                .contains("outside this proposal")
        );
        assert_eq!(std::fs::read(root.join("a.py")).unwrap(), b"external");
    }
    #[test]
    fn saved_and_draft_compiler_constructions_protect_missing_generated_destinations() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let entry = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"part.md\"/></h:doc>";
        let part = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"generated.py\">value = 1</h:file></h:doc>";
        std::fs::write(root.join("project.md"), entry).unwrap();
        std::fs::write(root.join("part.md"), part).unwrap();
        let mut request = fixture(
            json!({"documentChanges":[{"kind":"create","uri":"hick:///generated.py"},{"kind":"create","uri":"hick:///draft-generated.py"}]}),
        );
        request.ownership_sources.insert(
            "part.md".into(),
            part.replace("generated.py", "draft-generated.py"),
        );
        let inputs = capture_at(root, request).unwrap();
        for path in ["generated.py", "draft-generated.py"] {
            assert_eq!(inputs.saved_sources[path], SavedSource::Missing);
            assert!(matches!(
                inputs.sources[path],
                SourceState::Missing { creatable: false }
            ));
            assert!(!root.join(path).exists());
        }
        assert_eq!(inputs.ownership.entry.as_deref(), Some("project.md"));
        assert!(inputs.ownership.saved_sources.contains_key("part.md"));
        assert_ne!(
            inputs.ownership.saved_sources["part.md"],
            inputs.ownership.draft_sources["part.md"]
        );
    }
    #[test]
    fn directories_binary_files_and_missing_parents_never_become_editable_source_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        std::fs::create_dir(root.join("folder")).unwrap();
        std::fs::write(root.join("binary.py"), [255, 254]).unwrap();
        for path in ["folder", "binary.py"] {
            assert!(
                capture_at(
                    root,
                    fixture(json!({"changes":{(format!("hick:///{path}")):[]}}))
                )
                .is_err()
            );
        }
        let inputs = capture_at(
            root,
            fixture(json!({"documentChanges":[{"kind":"create","uri":"hick:///absent/new.py"}]})),
        )
        .unwrap();
        assert!(matches!(
            inputs.sources["absent/new.py"],
            SourceState::Missing { creatable: false }
        ));
        assert!(!root.join("absent").exists());
    }
    #[cfg(unix)]
    #[test]
    fn root_replacement_and_symbolic_links_are_refused() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("a.py"), "source").unwrap();
        let identity = RootIdentity::bind(&root);
        let id = identity.check(&root).unwrap().to_owned();
        std::fs::rename(&root, temp.path().join("original")).unwrap();
        std::fs::create_dir(&root).unwrap();
        assert!(identity.check(&root).is_err());
        assert!(!id.is_empty());
        std::os::unix::fs::symlink(temp.path().join("original/a.py"), root.join("link.py"))
            .unwrap();
        assert!(capture_at(&root, fixture(json!({"changes":{"hick:///link.py":[]}}))).is_err());
        assert_eq!(
            std::fs::read(temp.path().join("original/a.py")).unwrap(),
            b"source"
        );
    }
}
