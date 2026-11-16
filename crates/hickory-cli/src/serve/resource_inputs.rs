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
    /// Read-only retained outcomes remain addressable after a root is replaced.
    /// This does not establish present mutation authority; writers always call check.
    pub(super) fn retained_epoch(&self)->Result<&str,String>{match self.bound.get(){Some(Ok((_,_,epoch)))=>Ok(epoch.as_str()),Some(Err(error))=>Err(error.clone()),None=>Err("Original workspace epoch is not available yet.".into())}}
    /// Persistent reading identity is separate from this engine's random mutation epoch.
    /// Unsupported physical birth identity leaves guarded execution unavailable.
    pub fn fingerprint(root:&Path)->Result<String,String>{
        let canonical=root.canonicalize().map_err(|error|error.to_string())?;
        let held=same_file::Handle::from_path(&canonical).map_err(|error|error.to_string())?;
        let metadata=held.as_file().metadata().map_err(|error|error.to_string())?;
        if !metadata.is_dir(){return Err("Workspace identity requires an ordinary directory.".into())}
        let current=same_file::Handle::from_path(root).map_err(|error|error.to_string())?;
        if current!=held||root.canonicalize().map_err(|error|error.to_string())?!=canonical{return Err("Workspace changed while reading its physical identity.".into())}
        #[cfg(unix)] {let birth=metadata.created().map_err(|_|"Persistent workspace creation identity is unavailable on this filesystem; source remains readable but guarded execution needs a supported root identity.")?.duration_since(std::time::UNIX_EPOCH).map_err(|error|error.to_string())?.as_nanos().to_string();let bytes=serde_json::to_vec(&(canonical.as_os_str().as_encoded_bytes(),held.dev(),held.ino(),birth)).map_err(|error|error.to_string())?;Ok(sha(&bytes))}
        #[cfg(not(unix))] {let _=(held,metadata);Err("Persistent workspace reading identity is unavailable on this engine platform. Readable text remains available.".into())}
    }
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
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(default)]
    pub save_guard:Option<String>,
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
    #[serde(default)]
    pub unaddressed_sources: BTreeMap<String,String>,
    #[serde(default)]
    pub saved_guards:BTreeMap<String,String>,
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
    #[serde(default)]
    pub saved_guards:BTreeMap<String,String>,
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
/// Includes and reusable fragments can introduce generated ownership too.
/// Unknown parsing remains a refusal, never evidence that a draft is ordinary.
pub(crate) fn contains_construction(source:&str)->Result<bool,String>{let document=hick_lang::parse(source).map_err(|error|format!("Retained ownership cannot be analyzed: {error}. Preserve the source and select its construction context."))?;Ok(document.all_tags().iter().any(|tag|matches!(tag.name.as_str(),"project"|"file"|"entry"|"patch"|"scope"|"exec"|"volume"|"include"|"upstream"|"copy"|"paste")))}
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
            if contains_construction(source)?            {
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
        let canonical_request = Request {unaddressed_sources:request.unaddressed_sources.clone(),
            context: request.context.clone(),
            edit: request.edit.clone(),
            drafts: request.drafts.clone(),
            project_entry: None,
            ownership_sources: request.ownership_sources.clone(),
            saved_guards: request.saved_guards.clone(),
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
fn retained_ownership(root:&Path,request:&Request,ownership:&Ownership,sources:&BTreeMap<String,SourceState>)->Result<(),String>{
 let mut candidate=None;
 for text in request.unaddressed_sources.values(){if contains_construction(text)?{return Err("An unaddressed retained construction draft has no saved namespace authority. Preserve it, assign its authored source and select its construction before changing the namespace.".into())}}
 for(path,text)in &request.ownership_sources{
  if !contains_construction(text)?{continue}
  resource_changes::validate_path(path)?;
  if ownership.draft_sources.get(path)!=Some(&sha(text.as_bytes())){return Err(format!("Retained construction source {path} is outside the proven selected context. Preserve its draft and select its construction before changing the namespace."))}
  let(_,saved)=saved_target(root,path)?;
  match saved{
   SavedSource::File{text:saved,..}=>{if ownership.saved_sources.get(path)!=Some(&sha(saved.as_bytes())){return Err("Saved construction source changed outside its proven ownership context.".into())}},
   SavedSource::Missing=>{
    if candidate.is_none(){candidate=Some(resource_changes::prepare(resource_changes::Request{context:request.context.clone(),title:"Check new construction ownership".into(),edit:request.edit.clone(),sources:sources.clone()})?)}
    if !candidate.as_ref().unwrap().patches.iter().any(|patch|patch.path==*path&&matches!(&patch.before,SourceState::Missing{..})&&matches!(&patch.after,SourceState::File{text:after,..} if after==text)){return Err("A missing named construction must be created with exactly that text in the same complete selected proposal. No implicit file identity was supplied.".into())}
   }
  }
 }
 Ok(())
}
fn captured_file_guard(root:&Path,path:&str,saved:&SavedSource)->Result<String,String>{
 let(full,current)=saved_target(root,path)?;if &current!=saved{return Err(format!("Saved source changed while capturing its physical guard: {path}"))}
 let observed=super::source_history::read_observed(root,&full,path)?;let SavedSource::File{text,..}=saved else{return Err("Missing sources do not inherit an existing-file save guard.".into())};
 if &observed.source!=text{return Err(format!("Source changed while acquiring its exact physical observation: {path}"))}
 let guard=observed.save_guard.ok_or_else(||observed.save_guard_reason.unwrap_or("Physical source observation is unavailable.".into()))?;
 super::source_history::validate_observed_guard(root,&full,path,&guard)?;if read_saved(&full)?!=*saved{return Err(format!("Source permissions or bytes changed while capturing: {path}"))}Ok(guard)
}
pub(super) fn validate_guard_map(root:&Path,guards:&BTreeMap<String,String>,published:&BTreeSet<String>)->Result<(),String>{
 for(path,guard)in guards{if !published.contains(path){super::resource_changes::validate_path(path)?;super::source_history::validate_observed_guard(root,&root.join(path),path,guard)?}}Ok(())
}
fn capture(
    root: &Path,
    index: &super::store::DocIndex,
    identity: &RootIdentity,
    request: Request,
) -> Result<Inputs, String> {
    if request.saved_guards.len()>4096||request.saved_guards.values().any(|guard|guard.len()>16*1024)||request.saved_guards.values().map(String::len).sum::<usize>()>4*1024*1024{return Err("Captured physical source metadata exceeds its bounded capacity.".into())}
    if request.context.is_empty() || request.context.len() > 4096 {
        return Err(
            "Capture the current workspace/provider context before acquiring resource inputs."
                .into(),
        );
    }
    if request.ownership_sources.len().saturating_add(request.unaddressed_sources.len()) > 4096
        || request.unaddressed_sources.values().any(|source|source.len()>MAX_SOURCE)
        || request.unaddressed_sources.values().map(String::len).sum::<usize>().saturating_add(request.ownership_sources.values().map(String::len).sum::<usize>())>MAX_INPUT
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
    let (mut sources, mut saved) = (BTreeMap::new(), BTreeMap::new());let mut saved_guards=BTreeMap::new();
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
                let guard=captured_file_guard(&root,&path,&observed)?;
                let draft = request.drafts.get(&path);
                if let Some(draft)=draft {if draft.save_guard.as_deref()!=Some(guard.as_str()){return Err(format!("The editor has no matching originally loaded physical source guard: {path}. Preserve its draft, compare or reopen the exact source, then prepare again."))}}
                if let Some(provided)=request.saved_guards.get(&path){if provided!=&guard{return Err(format!("Retained source physical identity changed: {path}. No equal-byte replacement was adopted."))}}
                saved_guards.insert(path.clone(),guard);
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
    retained_ownership(&root,&request,&ownership,&sources)?;
    for(path,expected_hash)in &ownership.saved_sources{
        let(_,saved)=saved_target(&root,path)?;let SavedSource::File{sha256,..}=&saved else{return Err(format!("Saved construction input disappeared: {path}"))};if sha256!=expected_hash{return Err(format!("Saved construction bytes changed: {path}"))}
        let guard=if let Some(guard)=saved_guards.get(path){guard.clone()}else{captured_file_guard(&root,path,&saved)?};
        let provided=request.saved_guards.get(path).or_else(||request.drafts.get(path).and_then(|draft|draft.save_guard.as_ref()));
        if provided.is_some_and(|provided|provided!=&guard)||request.ownership_sources.contains_key(path)&&provided.is_none(){return Err(format!("Construction input has no matching captured physical source guard: {path}. Retain its draft and compare the exact saved source."))}
        saved_guards.insert(path.clone(),guard);
    }
    identity.check(&root)?;
    let result = Inputs {
        version: 1,
        context: request.context,
        workspace,
        ownership,
        sources,
        saved_sources: saved,
        saved_guards,
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
        Request {unaddressed_sources:BTreeMap::new(),
            context: "fixture".into(),
            edit,
            drafts: BTreeMap::new(),
            project_entry: None,
            ownership_sources: BTreeMap::new(),
            saved_guards: BTreeMap::new(),
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
                save_guard:super::super::source_history::read_observed(root,&root.join("a.py"),"a.py").unwrap().save_guard,
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
    fn originally_loaded_physical_guard_refuses_equal_byte_recreation_and_old_inputs(){
        let temp=tempfile::tempdir().unwrap();let root=temp.path();std::fs::write(root.join("a.py"),"same").unwrap();
        let loaded=super::super::source_history::read_observed(root,&root.join("a.py"),"a.py").unwrap().save_guard.unwrap();
        let mut request=fixture(json!({"changes":{"hick:///a.py":[]}}));request.drafts.insert("a.py".into(),Draft{text:"same".into(),base_sha256:sha(b"same"),version:None,writable:true,save_guard:None});
        assert!(capture_at(root,request.clone()).unwrap_err().contains("originally loaded"));request.drafts.get_mut("a.py").unwrap().save_guard=Some(loaded);
        let before=capture_at(root,request.clone()).unwrap();let unchanged=capture_at(root,request.clone()).unwrap();assert_eq!(before.saved_guards,unchanged.saved_guards);
        std::fs::rename(root.join("a.py"),root.join("original.py")).unwrap();std::fs::write(root.join("a.py"),"same").unwrap();assert!(capture_at(root,request.clone()).unwrap_err().contains("originally loaded"));
        request.drafts.clear();let independent=capture_at(root,request).unwrap();assert_eq!(before.saved_sources,independent.saved_sources);assert_ne!(before.saved_guards,independent.saved_guards);
        let plan=super::super::resource_changes::ResourcePlan{version:1,context:before.context.clone(),title:"Inspect physical revision".into(),patches:vec![],annotations:vec![]};assert!(validate_candidate(&before,&independent,&plan).is_err());assert_eq!(std::fs::read_to_string(root.join("a.py")).unwrap(),"same");assert!(!root.join(".hick-cache").exists());
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
                save_guard:super::super::source_history::read_observed(root,&root.join("a.py"),"a.py").unwrap().save_guard,
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
                save_guard:super::super::source_history::read_observed(root,&root.join("a.py"),"a.py").unwrap().save_guard,
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
        request.saved_guards.insert("part.md".into(),super::super::source_history::read_observed(root,&root.join("part.md"),"part.md").unwrap().save_guard.unwrap());
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
        request.saved_guards.insert("part.md".into(),super::super::source_history::read_observed(root,&root.join("part.md"),"part.md").unwrap().save_guard.unwrap());
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
        || captured.saved_guards != fresh.saved_guards
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
        Request {unaddressed_sources:BTreeMap::new(),
            context: "candidate".into(),
            edit,
            drafts: BTreeMap::new(),
            project_entry: None,
            ownership_sources: BTreeMap::new(),
            saved_guards: BTreeMap::new(),
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
    fn candidate_receipts_expose_engine_chosen_creation_and_move_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        project(root);
        std::fs::write(root.join("old.py"), "old\n").unwrap();
        let previous = saved_target(root, "old.py").unwrap().1;
        let request = request(json!({"documentChanges":[
            {"kind":"rename","oldUri":"hick:///old.py","newUri":"hick:///moved.py"},
            {"kind":"create","uri":"hick:///new.py"},
            text("new.py", "", "new\n")
        ]}));
        let facts = analyze(root, &request).unwrap();
        assert_eq!(facts.saved_sources["old.py"], SavedSource::Missing);
        assert_eq!(facts.saved_sources["moved.py"], previous);
        let SavedSource::File { text, sha256, mode, readonly, executable } = &facts.saved_sources["new.py"] else { panic!("missing saved candidate") };
        assert_eq!(text, "new\n");
        assert_eq!(sha256, &sha(text.as_bytes()));
        assert!(!readonly && !executable);
        assert_eq!(*mode, if cfg!(unix) { Some(0o600) } else { None });
        assert_eq!(facts.saved_sources.len(), 3);
        assert!(!root.join("new.py").exists());
        assert!(!root.join("moved.py").exists());
        assert_eq!(std::fs::read_to_string(root.join("old.py")).unwrap(), "old\n");
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
        request.saved_guards.insert("project.md".into(),super::super::source_history::read_observed(root,&root.join("project.md"),"project.md").unwrap().save_guard.unwrap());
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
        let request = Request {unaddressed_sources:BTreeMap::new(),
            context: "restore-layers".into(),
            edit: edit.clone(),
            drafts: BTreeMap::new(),
            project_entry: None,
            ownership_sources: BTreeMap::new(),
            saved_guards: BTreeMap::new(),
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
        assert_eq!(facts.saved_sources, projection);
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
#[cfg(test)] mod retained_ownership_tests{
 use super::*;
 #[cfg(unix)] #[test] fn persistent_root_fingerprint_survives_child_edits_but_not_same_path_recreation(){
  let parent=tempfile::tempdir().unwrap();let root=parent.path().join("workspace");std::fs::create_dir(&root).unwrap();std::fs::write(root.join("a.py"),"same bytes").unwrap();let original=RootIdentity::bind(&root);let before=RootIdentity::fingerprint(&root).unwrap();std::fs::write(root.join("b.py"),"child change").unwrap();assert_eq!(RootIdentity::fingerprint(&root).unwrap(),before);std::fs::rename(&root,parent.path().join("original")).unwrap();std::fs::create_dir(&root).unwrap();std::fs::write(root.join("a.py"),"same bytes").unwrap();std::fs::write(root.join("b.py"),"child change").unwrap();assert!(original.check(&root).is_err());let restarted=RootIdentity::bind(&root);restarted.check(&root).unwrap();assert_ne!(RootIdentity::fingerprint(&root).unwrap(),before);
 }

 #[test] fn complete_retained_roster_cannot_hide_construction_in_an_opaque_or_non_markdown_name(){
  let temporary=tempfile::tempdir().unwrap();let root=temporary.path();let entry="<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"part.md\"/></h:doc>";let part="<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"generated.py\">value = 1</h:file></h:doc>";
  for(path,text)in [("project.md",entry),("part.md",part),("a.py","saved"),("unselected.txt",part)]{std::fs::write(root.join(path),text).unwrap();}
  let index=super::super::store::DocIndex::scan(root).unwrap();let identity=RootIdentity::bind(root);
  let base=Request{unaddressed_sources:BTreeMap::from([("opaque-editor".into(),"Ordinary unnamed note".into())]),context:"retained-test".into(),edit:json!({"documentChanges":[{"kind":"rename","oldUri":"hick:///a.py","newUri":"hick:///b.py"}]}),drafts:BTreeMap::new(),project_entry:Some("project.md".into()),ownership_sources:BTreeMap::new(),saved_guards:BTreeMap::new()};
  assert!(capture(root,&index,&identity,base.clone()).is_ok());
  let mut unknown=base.clone();unknown.unaddressed_sources.insert("opaque-editor".into(),part.into());assert!(capture(root,&index,&identity,unknown).is_err());
  let mut include_only=base.clone();include_only.unaddressed_sources.insert("opaque-editor".into(),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:include from=\"part.md\"/></h:doc>".into());assert!(capture(root,&index,&identity,include_only).unwrap_err().contains("unaddressed"));
  for tag in ["include","upstream","copy","paste"]{assert!(contains_construction(&format!("<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:{tag}/></h:doc>")).unwrap())}
  let mut unselected=base.clone();unselected.ownership_sources.insert("unselected.txt".into(),part.into());assert!(capture(root,&index,&identity,unselected).is_err());
  let mut selected=base;selected.ownership_sources.insert("part.md".into(),part.into());selected.saved_guards.insert("part.md".into(),super::super::source_history::read_observed(root,&root.join("part.md"),"part.md").unwrap().save_guard.unwrap());assert!(capture(root,&index,&identity,selected).is_ok());assert_eq!(std::fs::read_to_string(root.join("a.py")).unwrap(),"saved");assert!(!root.join("b.py").exists());
 }
}
