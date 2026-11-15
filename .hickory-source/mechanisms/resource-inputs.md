<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Acquire resource proposal inputs without saving them

For maintainers connecting provider resource proposals to guarded saved-file operations.

## Keep saved and editor state in separate roles

Resource input acquisition returns facts for a later proposal. It neither applies a plan nor saves source. The saved representation retains exact text and permission metadata, while supplied drafts carry their own source/version and a saved-base SHA-256 value.

<ui:copy id="resource-input-imports">
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
</ui:copy>

## Load root identity independently of readable text

Hold the original directory handle and compare it with the current path before and after acquisition. A replaced directory at the same path is not the captured resource scope. Binding runs on a blocking worker; only this optional capability waits for readiness. Failure does not prevent bootstrap or raw file opening. Handle equality follows the host filesystem’s identity semantics; it is not a cryptographic filesystem attestation.

<ui:copy id="resource-input-root-identity">
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
</ui:copy>

## Describe source, saved baseline and ownership separately

The response binds observations to the supplied context and held root identity. Saved SHA-256 values are separate from ordinary file-save fingerprints. A draft must have the captured saved baseline; its contents do not implicitly save or replace that baseline. Limits apply to source bytes and the serialized response, including escaped text and repeated path keys.

<ui:copy id="resource-input-snapshots">
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub text: String,
    pub base_sha256: String,
    pub version: Option<i32>,
    pub writable: bool,
}
#[derive(Clone, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
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
</ui:copy>

## Acquire exact listed paths without following links

Inspect each component, refuse symbolic links, and require exact directory-entry spelling instead of silently accepting a case or Unicode alias. Cache bounded directory listings so multiple targets share that work. Missing parents stay unavailable rather than being created during acquisition. This is a read snapshot, not a filesystem freeze; publication must recheck its inputs.

<ui:copy id="resource-input-path-identity">
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
</ui:copy>

## Read regular UTF-8 source and modeled permissions

Distinguish confirmed NotFound from access failures. Refuse directories, special files and non-UTF-8 bytes instead of inventing empty source. Bound the read even if the file grows after metadata observation, and retain executable/read-only metadata alongside its exact bytes.

<ui:copy id="resource-input-saved-bytes">
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
</ui:copy>

## Represent the chosen ownership scope explicitly

Construction ownership uses the same read-only compiler snapshot as construction readings. Union outputs from saved and supplied draft construction inputs so an absent generated destination remains protected. Pin saved inputs while evaluating the draft. Without a selected construction, retain the existing saved declaration claims and permission metadata and label that narrower scope; draft construction declarations require an explicit entry.

Selecting a narrower construction preserves the main project's ownership claims.
Both saved and supplied draft constructions contribute protected destinations and
input hashes; inconsistent repeated observations refuse instead of blessing a
mixed snapshot. The response lists every construction entry used for protection.

<ui:copy id="resource-input-ownership-model">
/// Reobserve an exact ordinary source path without acquiring compiler context.
/// Publication callers still own generated, live-room and editor authority.
pub(super) fn saved_target(root: &Path, path: &str) -> Result<(PathBuf, SavedSource), String> {
    let mut names = Names {
        entries: BTreeMap::new(),
        count: 0,
        bytes: 0,
    };
    let full = confined(root, path, &mut names)?;
    let saved = read_saved(&full)?;
    Ok((full, saved))
}
fn default_project_entry(root: &Path) -> Result<Option<String>, String> {
    Ok(match read_saved(&root.join("project.md"))? {
        SavedSource::Missing => None,
        SavedSource::File { text, .. } => {
            let document=hick_lang::parse(&text).map_err(|error|format!("Construction entry cannot be parsed: {error}. Correct it before resource changes."))?;
            (!document.find_tags("project").is_empty()).then(|| "project.md".to_string())
        }
    })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ownership {
    pub mode: String,
    pub entry: Option<String>,
    pub protected_entries: Vec<String>,
    pub saved_sources: BTreeMap<String, String>,
    pub draft_sources: BTreeMap<String, String>,
}
fn ownership(
    root: &Path,
    index: &super::store::DocIndex,
    request: &Request,
) -> Result<(Ownership, BTreeSet<String>), String> {
    let canonical_entry = default_project_entry(root)?;
    let entry = if let Some(entry) = &request.project_entry {
        resource_changes::validate_path(entry)?;
        Some(entry.clone())
    } else {
        canonical_entry.clone()
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
                protected_entries: vec![],
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
    let mut saved_sources = before.source_hashes;
    let mut draft_sources = after.source_hashes;
    let mut protected_entries = vec![entry.clone()];
    if request.project_entry.is_some()
        && canonical_entry.as_deref().is_some_and(|root| root != entry)
    {
        // A selected reading context cannot remove the main project's ownership.
        let canonical_request = Request {
            context: request.context.clone(),
            edit: request.edit.clone(),
            drafts: request.drafts.clone(),
            project_entry: None,
            ownership_sources: request.ownership_sources.clone(),
        };
        let (canonical, protected) = ownership(root, index, &canonical_request)?;
        generated.extend(protected);
        for (current, extra) in [
            (&mut saved_sources, canonical.saved_sources),
            (&mut draft_sources, canonical.draft_sources),
        ] {
            for (path, hash) in extra {
                if current.get(&path).is_some_and(|previous| previous != &hash) {
                    return Err(
                        "Construction inputs changed during ownership acquisition. Prepare again."
                            .into(),
                    );
                }
                current.insert(path, hash);
            }
        }
        protected_entries.extend(canonical.protected_entries);
    }
    Ok((
        Ownership {
            mode: "compiler outputs from saved and supplied draft construction inputs".into(),
            entry: Some(entry),
            protected_entries,
            saved_sources,
            draft_sources,
        },
        generated,
    ))
}
</ui:copy>

## Combine saved observations with explicit draft overlays

Discover every source and destination before reading. Reject unrelated supplied drafts, stale editor bases, oversized inputs and unavailable targets as one acquisition. The resulting source capabilities intersect saved permissions, parent availability, generated ownership and the editor’s own availability flag. Return concrete refusal reasons with readable source facts; no file or directory is created.

<ui:copy id="resource-input-capture">
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
</ui:copy>

## Keep acquisition off the editor and verify live rooms

At most two blocking acquisitions run at once; unrelated routes do not share this gate. After the read, refuse any live document room that differs from the saved baseline, then recheck root identity. These are preconditions for preparing a proposal, not authorization to publish it. A later write must revalidate saved and editor revisions independently.

<ui:copy id="resource-input-independent-handler">
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
</ui:copy>

## Exercise saved/draft and ownership boundaries

Capture a dirty source without changing saved bytes or creating a destination. Check stale bases, unrelated drafts, binary/directory refusal, missing parents, saved/draft compiler ownership of absent outputs, symbolic links and root replacement. These witnesses cover acquisition, not frontend application or filesystem publication.

<ui:copy id="resource-input-read-witnesses">
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
    fn a_selected_construction_cannot_hide_main_project_outputs() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let entry = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"part.md\"/></h:doc>";
        let part = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"main-generated.py\">main</h:file></h:doc>";
        let other = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"other-generated.py\">other</h:file></h:doc>";
        std::fs::write(root.join("project.md"), entry).unwrap();
        std::fs::write(root.join("part.md"), part).unwrap();
        std::fs::write(
            root.join("other.md"),
            entry.replace("part.md", "other-part.md"),
        )
        .unwrap();
        std::fs::write(root.join("other-part.md"), other).unwrap();
        let mut request = fixture(json!({"documentChanges":[
            {"kind":"create","uri":"hick:///main-generated.py"},
            {"kind":"create","uri":"hick:///draft-generated.py"},
            {"kind":"create","uri":"hick:///other-generated.py"}
        ]}));
        request.project_entry = Some("other.md".into());
        request.ownership_sources.insert(
            "part.md".into(),
            part.replace("main-generated.py", "draft-generated.py"),
        );
        let inputs = capture_at(root, request).unwrap();
        assert_eq!(inputs.ownership.entry.as_deref(), Some("other.md"));
        assert_eq!(
            inputs.ownership.protected_entries,
            ["other.md", "project.md"]
        );
        for path in [
            "main-generated.py",
            "draft-generated.py",
            "other-generated.py",
        ] {
            assert!(matches!(
                inputs.sources[path],
                SourceState::Missing { creatable: false }
            ));
            assert!(!root.join(path).exists());
        }
        for path in ["project.md", "part.md", "other.md", "other-part.md"] {
            assert!(inputs.ownership.saved_sources.contains_key(path));
            assert!(inputs.ownership.draft_sources.contains_key(path));
        }
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
</ui:copy>

## Revalidate a complete candidate against fresh observations

Fresh acquisition intentionally allocates new draft identities. Revalidation compares source bytes, version stamps and capabilities while keeping the reviewed plan’s identities for its ordered namespace transition. Exact saved states, held root scope and ownership snapshots must still agree. This returns a new namespace only; it authorizes no filesystem publication.

<ui:copy id="resource-input-fresh-candidate">
pub(crate) fn validate_candidate(
    captured: &Inputs,
    fresh: &Inputs,
    plan: &super::resource_changes::ResourcePlan,
) -> Result<BTreeMap<String, SourceState>, String> {
    fn same(a: &SourceState, b: &SourceState) -> bool {
        match (a, b) {
            (SourceState::Missing { creatable: a }, SourceState::Missing { creatable: b }) => {
                a == b
            }
            (
                SourceState::File {
                    text: a,
                    writable: aw,
                    removable: ar,
                    version: av,
                    ..
                },
                SourceState::File {
                    text: b,
                    writable: bw,
                    removable: br,
                    version: bv,
                    ..
                },
            ) => a == b && aw == bw && ar == br && av == bv,
            _ => false,
        }
    }
    if captured.version != 1
        || fresh.version != 1
        || captured.source_files_modified
        || fresh.source_files_modified
        || captured.workspace != fresh.workspace
        || captured.context != fresh.context
        || captured.saved_sources != fresh.saved_sources
        || captured.ownership != fresh.ownership
        || captured.sources.len() != fresh.sources.len()
        || !captured.sources.iter().all(|(path, state)| {
            fresh
                .sources
                .get(path)
                .is_some_and(|value| same(state, value))
        })
    {
        return Err("Saved source, workspace identity, construction ownership or draft acquisition changed. Prepare the whole resource proposal again.".into());
    }
    let after = plan.apply(&fresh.context, &captured.sources)?;
    for patch in &plan.patches {
        if !matches!(
            fresh.sources.get(&patch.path),
            Some(SourceState::File { writable: true, .. })
                | Some(SourceState::Missing { creatable: true })
        ) {
            return Err(format!(
                "Resource target is unavailable for publication: {}. Prepare again without unavailable targets.",
                patch.path
            ));
        }
    }
    Ok(after)
}
</ui:copy>

## Exercise final namespace ownership before any publication

Analyze source changes that introduce a generated destination and refuse direct writes to it. Follow an entry identity through a move; refuse a removed dependency and an unowned construction fragment. Permit a fragment owned by the candidate draft layer even when the saved layer has not linked it yet. Fresh draft identities may change, but source bytes, root scope, capabilities and saved ownership remain exact. No source or generated output is created by these checks.

<ui:copy id="resource-candidate-ownership-tests">
#[cfg(test)]
mod candidate_tests {
    use super::*;
    use crate::serve::{resource_candidates, resource_changes};
    fn project(root: &Path) -> (String, String) {
        let entry="<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"part.md\"/></h:doc>".to_string();
        let part="<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"before.py\">value = 1</h:file></h:doc>".to_string();
        std::fs::write(root.join("project.md"), &entry).unwrap();
        std::fs::write(root.join("part.md"), &part).unwrap();
        (entry, part)
    }
    fn request(edit: Value) -> Request {
        Request {
            context: "candidate".into(),
            edit,
            drafts: BTreeMap::new(),
            project_entry: None,
            ownership_sources: BTreeMap::new(),
        }
    }
    fn text(path: &str, before: &str, after: &str) -> Value {
        json!({"textDocument":{"uri":format!("hick:///{path}"),"version":null},"edits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":before.encode_utf16().count()}},"newText":after}]})
    }
    fn analyze(root: &Path, request: &Request) -> Result<resource_candidates::Facts, String> {
        let index = crate::serve::store::DocIndex::scan(root).unwrap();
        let identity = RootIdentity::bind(root);
        let captured = capture(root, &index, &identity, request.clone())?;
        let fresh = capture(root, &index, &identity, request.clone())?;
        let plan = resource_changes::prepare(resource_changes::Request {
            context: request.context.clone(),
            title: "Candidate namespace".into(),
            edit: request.edit.clone(),
            sources: captured.sources.clone(),
        })?;
        resource_candidates::analyze(root, &captured, &fresh, request, &plan)
    }
    #[test]
    fn candidate_generated_destinations_are_protected_before_source_publication() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let (_, part) = project(root);
        let updated = part.replace("before.py", "after.py");
        let request = request(
            json!({"documentChanges":[text("part.md",&part,&updated),{"kind":"create","uri":"hick:///after.py"}]}),
        );
        let facts = analyze(root, &request).unwrap();
        assert_eq!(facts.protected_targets, ["after.py"]);
        assert!(facts.saved.output_paths.contains("after.py"));
        assert!(facts.draft.output_paths.contains("after.py"));
        assert_ne!(
            facts.checked_saved_sources["part.md"],
            facts.saved.source_hashes["part.md"]
        );
        assert!(!facts.source_files_modified);
        assert_eq!(std::fs::read_to_string(root.join("part.md")).unwrap(), part);
        assert!(!root.join("after.py").exists());
    }
    #[test]
    fn source_changes_can_change_constructed_outputs_without_writing_them() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let (_, part) = project(root);
        let updated = part.replace("before.py", "after.py");
        let request = request(json!({"documentChanges":[text("part.md",&part,&updated)]}));
        let facts = analyze(root, &request).unwrap();
        assert!(facts.protected_targets.is_empty());
        assert!(facts.saved.output_paths.contains("after.py"));
        assert!(!facts.saved.output_paths.contains("before.py"));
        assert!(!root.join("after.py").exists());
    }
    #[test]
    fn entry_identity_follows_a_move_and_broken_removed_dependencies_refuse() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        project(root);
        let moved = request(
            json!({"documentChanges":[{"kind":"rename","oldUri":"hick:///project.md","newUri":"hick:///moved.md"}]}),
        );
        let facts = analyze(root, &moved).unwrap();
        assert!(facts.protected_targets.is_empty());
        assert_eq!(facts.saved.entries, ["moved.md"]);
        assert!(facts.saved.output_paths.contains("before.py"));
        assert!(root.join("project.md").exists());
        assert!(!root.join("moved.md").exists());
        let deleted =
            request(json!({"documentChanges":[{"kind":"delete","uri":"hick:///part.md"}]}));
        assert!(
            analyze(root, &deleted)
                .unwrap_err()
                .contains("deleted source part.md")
        );
    }
    #[test]
    fn a_new_fragment_needs_one_candidate_project_context() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let (entry, part) = project(root);
        let newpart = part.replace("before.py", "new.py");
        let edit = json!({"documentChanges":[{"kind":"create","uri":"hick:///new.md"},text("new.md","",&newpart)]});
        let mut request = request(edit);
        assert!(
            analyze(root, &request)
                .unwrap_err()
                .contains("no selected project context")
        );
        request
            .ownership_sources
            .insert("project.md".into(), entry.replace("part.md", "new.md"));
        request.ownership_sources.insert("new.md".into(), newpart);
        let facts = analyze(root, &request).unwrap();
        assert!(facts.protected_targets.is_empty());
        assert!(!facts.saved.source_hashes.contains_key("new.md"));
        assert!(facts.draft.source_hashes.contains_key("new.md"));
        assert!(facts.draft.output_paths.contains("new.py"));
        assert!(!root.join("new.md").exists());
    }
    #[test]
    fn fresh_capture_identity_is_independent_but_saved_root_and_source_facts_are_exact() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        std::fs::write(root.join("a.py"), "a").unwrap();
        let request = request(
            json!({"documentChanges":[{"kind":"rename","oldUri":"hick:///a.py","newUri":"hick:///b.py"}]}),
        );
        let index = crate::serve::store::DocIndex::scan(root).unwrap();
        let identity = RootIdentity::bind(root);
        let captured = capture(root, &index, &identity, request.clone()).unwrap();
        let mut fresh = capture(root, &index, &identity, request.clone()).unwrap();
        let plan = resource_changes::prepare(resource_changes::Request {
            context: request.context.clone(),
            title: "Move source".into(),
            edit: request.edit.clone(),
            sources: captured.sources.clone(),
        })
        .unwrap();
        assert_ne!(captured.sources, fresh.sources);
        assert!(validate_candidate(&captured, &fresh, &plan).is_ok());
        let first = resource_candidates::analyze(root, &captured, &fresh, &request, &plan).unwrap();
        let second =
            resource_candidates::analyze(root, &captured, &fresh, &request, &plan).unwrap();
        assert_eq!(first.fingerprint, second.fingerprint);
        fresh.workspace = "other-root".into();
        assert!(validate_candidate(&captured, &fresh, &plan).is_err());
        let fresh = capture(root, &index, &identity, request.clone()).unwrap();
        std::fs::write(root.join("a.py"), "external").unwrap();
        let changed = capture(root, &index, &identity, request).unwrap();
        assert!(validate_candidate(&captured, &changed, &plan).is_err());
        assert!(
            validate_candidate(&captured, &fresh, &plan).is_ok(),
            "A captured observation is not a filesystem freeze; publication must recheck it"
        );
    }
    #[test]
    fn new_project_roots_add_ownership_and_removed_entries_do_not_resurrect() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let (entry, part) = project(root);
        let new_entry = entry.replace("part.md", "new-part.md");
        let new_part = part.replace("before.py", "created.py");
        let new = request(json!({"documentChanges":[
            {"kind":"create","uri":"hick:///new-project.md"},text("new-project.md","",&new_entry),
            {"kind":"create","uri":"hick:///new-part.md"},text("new-part.md","",&new_part),
            {"kind":"create","uri":"hick:///created.py"}
        ]}));
        let facts = analyze(root, &new).unwrap();
        assert_eq!(facts.protected_targets, ["created.py"]);
        assert_eq!(facts.saved.entries, ["new-project.md", "project.md"]);
        assert!(facts.saved.source_hashes.contains_key("new-part.md"));
        assert!(!root.join("new-project.md").exists());
        let removed =
            request(json!({"documentChanges":[{"kind":"delete","uri":"hick:///project.md"}]}));
        let facts = analyze(root, &removed).unwrap();
        assert!(facts.saved.entries.is_empty());
        assert_eq!(facts.saved.removed_entries, ["project.md"]);
        assert!(root.join("project.md").exists());
    }
    #[test]
    fn an_unavailable_target_cannot_be_republished_as_a_nominal_noop() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let path = root.join("a.py");
        std::fs::write(&path, "readable").unwrap();
        let original = std::fs::metadata(&path).unwrap().permissions();
        let mut readonly = original.clone();
        readonly.set_readonly(true);
        std::fs::set_permissions(&path, readonly).unwrap();
        let request = request(json!({"changes":{"hick:///a.py":[]}}));
        let index = crate::serve::store::DocIndex::scan(root).unwrap();
        let identity = RootIdentity::bind(root);
        let captured = capture(root, &index, &identity, request.clone()).unwrap();
        let fresh = capture(root, &index, &identity, request).unwrap();
        let state = captured.sources["a.py"].clone();
        let plan = resource_changes::ResourcePlan {
            version: 1,
            context: captured.context.clone(),
            title: "Unchanged source".into(),
            patches: vec![resource_changes::ResourcePatch {
                path: "a.py".into(),
                before: state.clone(),
                after: state,
            }],
            annotations: vec![],
        };
        let pure = plan.apply(&captured.context, &captured.sources);
        let admission = validate_candidate(&captured, &fresh, &plan);
        std::fs::set_permissions(&path, original).unwrap();
        assert!(
            pure.is_ok(),
            "A pure namespace no-op does not authorize a filesystem replacement"
        );
        assert!(admission.is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"readable");
    }
}
</ui:copy>

## Analyze restored saved source independently of its dirty draft

Build one reviewed draft that declares a draft-only output and a separate saved restoration that declares a saved-only output. Both destinations must be protected in the same response, with each compiler layer retaining its own hashes and output set. Incomplete saved projections, contradictory absence and false saved fingerprints refuse. All analysis leaves the saved source and proposed destinations untouched.

<ui:copy id="restored-saved-draft-candidate-test">
#[cfg(test)]
mod restored_saved_candidate_tests {
    use super::*;
    #[test]
    fn restored_saved_source_and_unsaved_draft_keep_separate_compiler_layers() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let entry = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"part.md\"/></h:doc>";
        let original = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"original.py\">value = 1</h:file></h:doc>";
        let draft = original.replace("original.py", "draft-output.py");
        let saved = original.replace("original.py", "saved-output.py");
        std::fs::write(root.join("project.md"), entry).unwrap();
        std::fs::write(root.join("part.md"), original).unwrap();
        let edit = json!({"documentChanges":[
            {"textDocument":{"uri":"hick:///part.md","version":null},"edits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":original.encode_utf16().count()}},"newText":draft}]},
            {"kind":"create","uri":"hick:///saved-output.py"},
            {"kind":"create","uri":"hick:///draft-output.py"}
        ]});
        let request = Request {
            context: "restore-layers".into(),
            edit: edit.clone(),
            drafts: BTreeMap::new(),
            project_entry: None,
            ownership_sources: BTreeMap::new(),
        };
        let identity = RootIdentity::bind(root);
        let index = super::super::store::DocIndex::scan(root).unwrap();
        let inputs = capture(root, &index, &identity, request.clone()).unwrap();
        let fresh = capture(root, &index, &identity, request.clone()).unwrap();
        let plan =
            super::super::resource_changes::prepare(super::super::resource_changes::Request {
                context: request.context.clone(),
                title: "Restore two layers".into(),
                edit,
                sources: inputs.sources.clone(),
            })
            .unwrap();
        let ordinary =
            super::super::resource_candidates::analyze(root, &inputs, &fresh, &request, &plan)
                .unwrap();
        let mut projection = super::super::resource_journal::Manifest::from_review(&inputs, &plan)
            .unwrap()
            .after;
        if let SavedSource::File { text, sha256, .. } = projection.get_mut("part.md").unwrap() {
            *text = saved.clone();
            *sha256 = sha(saved.as_bytes());
        }
        let facts = super::super::resource_candidates::analyze_with_saved(
            root,
            &inputs,
            &fresh,
            &request,
            &plan,
            Some(&projection),
        )
        .unwrap();
        assert!(facts.saved.output_paths.contains("saved-output.py"));
        assert!(!facts.saved.output_paths.contains("draft-output.py"));
        assert!(facts.draft.output_paths.contains("draft-output.py"));
        assert!(!facts.draft.output_paths.contains("saved-output.py"));
        assert_eq!(
            facts.protected_targets,
            ["draft-output.py", "saved-output.py"]
        );
        assert_eq!(facts.saved.source_hashes["part.md"], sha(saved.as_bytes()));
        assert_eq!(facts.draft.source_hashes["part.md"], sha(draft.as_bytes()));
        assert_ne!(facts.fingerprint, ordinary.fingerprint);

        // A saved layer consuming older text cannot legitimize an unrelated,
        // unconsumed draft fragment at the same address.
        let mut detached = request.clone();
        detached.edit["documentChanges"].as_array_mut().unwrap().push(json!({
            "textDocument":{"uri":"hick:///project.md","version":null},
            "edits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":entry.encode_utf16().count()}},"newText":entry.replace("part.md", "")}]
        }));
        let detached_inputs = capture(root, &index, &identity, detached.clone()).unwrap();
        let detached_fresh = capture(root, &index, &identity, detached.clone()).unwrap();
        let detached_plan =
            super::super::resource_changes::prepare(super::super::resource_changes::Request {
                context: detached.context.clone(),
                title: "Separate consumed text".into(),
                edit: detached.edit.clone(),
                sources: detached_inputs.sources.clone(),
            })
            .unwrap();
        let mut detached_saved =
            super::super::resource_journal::Manifest::from_review(&detached_inputs, &detached_plan)
                .unwrap()
                .after;
        detached_saved.insert(
            "project.md".into(),
            detached_inputs.saved_sources["project.md"].clone(),
        );
        detached_saved.insert(
            "part.md".into(),
            detached_inputs.saved_sources["part.md"].clone(),
        );
        assert!(
            super::super::resource_candidates::analyze_with_saved(
                root,
                &detached_inputs,
                &detached_fresh,
                &detached,
                &detached_plan,
                Some(&detached_saved)
            )
            .unwrap_err()
            .contains("no selected project context")
        );

        let mut incomplete = projection.clone();
        incomplete.remove("part.md");
        assert!(
            super::super::resource_candidates::analyze_with_saved(
                root,
                &inputs,
                &fresh,
                &request,
                &plan,
                Some(&incomplete)
            )
            .is_err()
        );
        let mut missing = projection.clone();
        missing.insert("part.md".into(), SavedSource::Missing);
        assert!(
            super::super::resource_candidates::analyze_with_saved(
                root,
                &inputs,
                &fresh,
                &request,
                &plan,
                Some(&missing)
            )
            .is_err()
        );
        if let SavedSource::File { sha256, .. } = projection.get_mut("part.md").unwrap() {
            *sha256 = "wrong".into();
        }
        assert!(
            super::super::resource_candidates::analyze_with_saved(
                root,
                &inputs,
                &fresh,
                &request,
                &plan,
                Some(&projection)
            )
            .is_err()
        );
        assert_eq!(
            std::fs::read_to_string(root.join("part.md")).unwrap(),
            original
        );
        assert!(!root.join("saved-output.py").exists());
        assert!(!root.join("draft-output.py").exists());
    }
}
</ui:copy>

<ui:file path="crates/hickory-cli/src/serve/resource_inputs.rs"><ui:paste select="#resource-input-imports"/><ui:paste select="#resource-input-root-identity"/><ui:paste select="#resource-input-snapshots"/><ui:paste select="#resource-input-path-identity"/><ui:paste select="#resource-input-saved-bytes"/><ui:paste select="#resource-input-ownership-model"/><ui:paste select="#resource-input-capture"/><ui:paste select="#resource-input-independent-handler"/><ui:paste select="#resource-input-read-witnesses"/><ui:paste select="#resource-input-fresh-candidate"/><ui:paste select="#resource-candidate-ownership-tests"/><ui:paste select="#restored-saved-draft-candidate-test"/></ui:file>
## Exercise the actual acquisition and planning routes

Start a temporary real engine with one saved source and supply a separate dirty draft. Discover both move endpoints, acquire their saved/draft states, and prepare the ordered move/text proposal. Check the unchanged saved file and absent destination, then require an HTTP refusal for a stale editor base. After those read-only assertions, explicitly save the reviewed move and check exact saved/draft recovery, duplicate request identity, read-only status, generated-destination and stale-source refusal, and authored-document retirement. The witness exercises source publication through the real engine; it does not establish frontend namespace application or saved/draft undo.

<ui:copy id="resource-input-actual-service-witness">
"""Actual engine protocol: read-only preparation followed by explicit reviewed resource saves."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import urllib.error
import urllib.request
ROOT=Path(__file__).resolve().parents[1]
def main():
 spec=importlib.util.spec_from_file_location("fixture",ROOT/"scripts/test-simple-browser.py");fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix="resource-service-") as temporary:
  folder=Path(temporary);root=folder/"workspace";root.mkdir();saved="saved\n";draft=saved+"# unsaved draft\n";(root/"a.py").write_bytes(saved.encode());engine=None
  entry='<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources="part.md"/></h:doc>'
  part='<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:file path="main-generated.py">main</h:file></h:doc>'
  (root/"project.md").write_text(entry);(root/"part.md").write_text(part)
  (root/"other.md").write_text(entry.replace("part.md","other-part.md"));(root/"other-part.md").write_text(part.replace("main-generated.py","other-generated.py"))
  try:
   engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window","http://127.0.0.1:1"],env={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")});origin=fixture.origin(engine)
   def call(route,body):
    request=urllib.request.Request(origin+"/api/"+route,data=json.dumps(body,ensure_ascii=False).encode(),headers={"Content-Type":"application/json"},method="POST")
    with urllib.request.urlopen(request,timeout=30) as reply:return json.load(reply)
   edit={"documentChanges":[{"kind":"rename","oldUri":"hick:///a.py","newUri":"hick:///b.py"},{"textDocument":{"uri":"hick:///b.py","version":7},"edits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"newText":"renamed"}]}]}
   targets=call("changes/resource-targets",{"edit":edit});assert targets["paths"]==["a.py","b.py"] and not targets["acquired"]
   body={"context":"service-fixture","edit":edit,"drafts":{"a.py":{"text":draft,"base_sha256":hashlib.sha256(saved.encode()).hexdigest(),"version":7,"writable":True}}}
   inputs=call("changes/resource-inputs",body);assert inputs["context"]==body["context"] and inputs["workspace"] and not inputs["source_files_modified"];assert inputs["sources"]["a.py"]["text"]==draft;assert inputs["saved_sources"]["a.py"]["text"]==saved;assert inputs["saved_sources"]["b.py"]["kind"]=="missing"
   proposal=call("changes/plan-resources",{"context":body["context"],"title":"Move draft","edit":edit,"sources":inputs["sources"]});assert not proposal["applied"] and not proposal["saved"];patches={patch["path"]:patch for patch in proposal["plan"]["patches"]};assert patches["a.py"]["after"]["kind"]=="missing";assert patches["b.py"]["after"]["text"]=="renamed\n# unsaved draft\n";assert patches["b.py"]["after"]["identity"]==inputs["sources"]["a.py"]["identity"]
   consequences=call("changes/resource-consequences",{"inputs":inputs,"capture":body,"plan":proposal["plan"]});assert consequences["protected_targets"]==[] and not consequences["source_files_modified"] and len(consequences["fingerprint"])==64
   reviewed_save=json.loads(json.dumps({"request_id":"1"*32,"reviewed_fingerprint":consequences["fingerprint"],"proposal":{"inputs":inputs,"capture":body,"plan":proposal["plan"]}}))
   candidate_part=part.replace("main-generated.py","candidate-generated.py")
   edit2={"documentChanges":[{"textDocument":{"uri":"hick:///part.md","version":None},"edits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":len(part.encode("utf-16-le"))//2}},"newText":candidate_part}]},{"kind":"create","uri":"hick:///candidate-generated.py"}]}
   capture2={"context":"candidate-owned-output","edit":edit2};inputs2=call("changes/resource-inputs",capture2);plan2=call("changes/plan-resources",{"context":capture2["context"],"title":"Candidate output guard","edit":edit2,"sources":inputs2["sources"]})["plan"]
   consequences2=call("changes/resource-consequences",{"inputs":inputs2,"capture":capture2,"plan":plan2});assert consequences2["protected_targets"]==["candidate-generated.py"] and "candidate-generated.py" in consequences2["saved"]["output_paths"] and "candidate-generated.py" in consequences2["draft"]["output_paths"]
   assert (root/"part.md").read_text()==part and not (root/"candidate-generated.py").exists()
   body["drafts"]["a.py"]["base_sha256"]=hashlib.sha256(b"old").hexdigest()
   try:call("changes/resource-inputs",body)
   except urllib.error.HTTPError as error:assert error.code==422
   else:raise AssertionError("Stale saved base was accepted")
   protected=["main-generated.py","draft-generated.py","other-generated.py"]
   inputs=call("changes/resource-inputs",{"context":"narrow-reading","project_entry":"other.md","ownership_sources":{"part.md":part.replace("main-generated.py","draft-generated.py")},"edit":{"documentChanges":[{"kind":"create","uri":"hick:///"+path} for path in protected]}})
   assert inputs["ownership"]["protected_entries"]==["other.md","project.md"]
   for path in protected:assert inputs["sources"][path]=={"kind":"missing","creatable":False} and not (root/path).exists()
   try:call("construction/reading",{"entry":"project.md","sources":{},"missing_sources":["part.md"]})
   except urllib.error.HTTPError as error:assert error.code==422
   else:raise AssertionError("Deleted construction source fell back to saved bytes")
   reading=call("construction/reading",{"entry":"project.md","sources":{"project.md":entry.replace("part.md","moved.md"),"moved.md":part.replace(">main<",">moved<")},"missing_sources":["part.md"]})
   output=next(item for item in reading["outputs"] if item["path"]=="main-generated.py");assert output["before"]["preview"]=="main" and output["after"]["preview"]=="moved"
   assert (root/"part.md").read_text()==part and (root/"project.md").read_text()==entry and not (root/"moved.md").exists()
   assert (root/"a.py").read_bytes()==saved.encode();assert not (root/"b.py").exists()
   records=root/".hick-cache"/"resource-changes"
   existing=set(records.iterdir()) if records.exists() else set()
   rejected={"request_id":"2"*32,"reviewed_fingerprint":consequences2["fingerprint"],"proposal":{"inputs":inputs2,"capture":capture2,"plan":plan2}}
   try:call("changes/resource-save",rejected)
   except urllib.error.HTTPError as error:assert error.code==409
   else:raise AssertionError("Candidate generated destination was published")
   assert not (root/"candidate-generated.py").exists() and (root/"part.md").read_text()==part
   saved_result=call("changes/resource-save",reviewed_save);outcome=saved_result["outcome"]
   assert outcome["complete"] and outcome["source_files_modified"] and not outcome["source_files_uncertain"]
   assert outcome["applied"]==["a.py","b.py"]
   assert not (root/"a.py").exists() and (root/"b.py").read_text()=="renamed\n# unsaved draft\n"
   record=json.loads((root/outcome["recovery"]).read_text());assert record["version"]==2 and record["before"]["a.py"]["text"]==saved
   assert next(patch for patch in record["reviewed_plan"]["patches"] if patch["path"]=="a.py")["before"]["text"]==draft
   after=set(records.iterdir());assert len(after-existing)==1
   assert call("changes/resource-save",reviewed_save)==saved_result and set(records.iterdir())==after
   with urllib.request.urlopen(origin+"/api/changes/resource-save?request_id="+reviewed_save["request_id"],timeout=30) as reply:assert json.load(reply)==saved_result
   changed=json.loads(json.dumps(reviewed_save));changed["reviewed_fingerprint"]="0"*64
   try:call("changes/resource-save",changed)
   except urllib.error.HTTPError as error:assert error.code==409
   else:raise AssertionError("A save request identity accepted different review content")
   assert set(records.iterdir())==after
   stale_edit={"documentChanges":[{"kind":"rename","oldUri":"hick:///b.py","newUri":"hick:///c.py"}]}
   stale_capture={"context":"stale-save","edit":stale_edit};stale_inputs=call("changes/resource-inputs",stale_capture)
   stale_plan=call("changes/plan-resources",{"context":stale_capture["context"],"title":"Stale move","edit":stale_edit,"sources":stale_inputs["sources"]})["plan"]
   stale_proposal={"inputs":stale_inputs,"capture":stale_capture,"plan":stale_plan};stale_facts=call("changes/resource-consequences",stale_proposal)
   (root/"b.py").write_text("independent edit\n")
   try:call("changes/resource-save",{"request_id":"3"*32,"reviewed_fingerprint":stale_facts["fingerprint"],"proposal":stale_proposal})
   except urllib.error.HTTPError as error:assert error.code==409
   else:raise AssertionError("Stale save changed a source")
   assert (root/"b.py").read_text()=="independent edit\n" and not (root/"c.py").exists() and set(records.iterdir())==after
   note='<h:doc xmlns:h="http://www.hickorydocs.com/1.0">Original note</h:doc>'
   (root/"note.md").write_text(note)
   note_edit={"documentChanges":[{"kind":"rename","oldUri":"hick:///note.md","newUri":"hick:///moved-note.md"}]}
   note_capture={"context":"move-authored-document","edit":note_edit};note_inputs=call("changes/resource-inputs",note_capture)
   note_plan=call("changes/plan-resources",{"context":note_capture["context"],"title":"Move note","edit":note_edit,"sources":note_inputs["sources"]})["plan"]
   note_proposal={"inputs":note_inputs,"capture":note_capture,"plan":note_plan};note_facts=call("changes/resource-consequences",note_proposal)
   note_result=call("changes/resource-save",{"request_id":"4"*32,"reviewed_fingerprint":note_facts["fingerprint"],"proposal":note_proposal})
   assert note_result["outcome"]["complete"] and not (root/"note.md").exists() and (root/"moved-note.md").read_text()==note,note_result
   root_edit={"documentChanges":[{"kind":"rename","oldUri":"hick:///project.md","newUri":"hick:///moved-project.md"}]}
   root_capture={"context":"canonical-entry-save-guard","edit":root_edit};root_inputs=call("changes/resource-inputs",root_capture)
   root_plan=call("changes/plan-resources",{"context":root_capture["context"],"title":"Move canonical entry","edit":root_edit,"sources":root_inputs["sources"]})["plan"]
   root_proposal={"inputs":root_inputs,"capture":root_capture,"plan":root_plan};root_facts=call("changes/resource-consequences",root_proposal)
   root_records=set(records.iterdir())
   try:call("changes/resource-save",{"request_id":"5"*32,"reviewed_fingerprint":root_facts["fingerprint"],"proposal":root_proposal})
   except urllib.error.HTTPError as error:assert error.code==409 and "canonical project.md" in error.read().decode()
   else:raise AssertionError("Resource save silently removed canonical entry ownership")
   assert (root/"project.md").read_text()==entry and not (root/"moved-project.md").exists() and set(records.iterdir())==root_records
   print("PASS: actual resource acquisition, planning and candidate ownership; explicit guarded save of a moved dirty draft, retained saved/draft recovery, duplicate/status identity, changed-request refusal, stale-source and generated-destination refusal, authored document retirement/move, canonical-entry ownership guard")
  finally:
   if engine:fixture.stop(engine)
if __name__=="__main__":main()
</ui:copy>

<ui:file path="scripts/test-resource-proposal-service.py"><ui:paste select="#resource-input-actual-service-witness"/></ui:file>
</ui:doc>
