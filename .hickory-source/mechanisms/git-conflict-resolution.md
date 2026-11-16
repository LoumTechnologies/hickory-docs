<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Resolve an explicitly selected conflict group

For maintainers extending conflict resolution without separating Git index authority from source ownership. Git stages are immutable readings, editable source is a separate working namespace, and staging is a third explicit act. A selected group never implies a guessed rename mapping: its retained destinations and source identities are explicit values.

## Capture the actual repository index

Use Git's NUL-delimited index protocol and literal paths. Bind the whole index and merge heads, including unrelated entries, while exposing only workspace paths. Hold the real index lock during publication or complete-group staging; never remove another process's lock. A temporary index carries all group removals and resolved entries until every precondition passes, then one rename installs it. Unsupported source namespace spellings remain readable Git observations and report publication refusal rather than bypassing the shared codec.

<ui:copy id="git-resolution-index">
//! Reviewed source conflict groups share source publication and one atomic Git index act.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
    resource_changes::{ResourcePatch, ResourcePlan, SourceState},
    resource_inputs::{Inputs, SavedSource},
};
use axum::{
    Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
};
const MAX_INDEX: usize = 64 * 1024 * 1024;
fn hash(bytes: &[u8]) -> String {
    super::workspace_fs::view::hash(bytes)
}
fn nonce() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|e| e.to_string())?;
    Ok(hex::encode(bytes))
}
fn command(root: &Path) -> Command {
    let mut c = Command::new("git");
    c.arg("-C")
        .arg(root)
        .arg("--literal-pathspecs")
        .arg("-c")
        .arg("core.fsmonitor=false")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE");
    c
}
fn git(
    root: &Path,
    args: &[&str],
    input: Option<&[u8]>,
    index: Option<&Path>,
) -> Result<Vec<u8>, String> {
    let mut c = command(root);
    c.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(path) = index {
        c.env("GIT_INDEX_FILE", path);
    }
    if input.is_some() {
        c.stdin(Stdio::piped());
    }
    let mut child = c.spawn().map_err(|e| format!("Git is unavailable: {e}"))?;
    if let Some(data) = input {
        child
            .stdin
            .take()
            .ok_or("Git input unavailable")?
            .write_all(data)
            .map_err(|e| e.to_string())?;
    }
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "Git refused: {}",
            String::from_utf8_lossy(&output.stderr)
                .chars()
                .take(2000)
                .collect::<String>()
        ));
    }
    if output.stdout.len() > MAX_INDEX {
        return Err("Git observation exceeds its bounded interactive index budget.".into());
    }
    Ok(output.stdout)
}
fn git_path(root: &Path, args: &[&str]) -> Result<PathBuf, String> {
    let mut bytes = git(root, args, None, None)?;
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
    }
    let text = String::from_utf8(bytes)
        .map_err(|_| "Git repository paths must be UTF-8 for this provider")?;
    let path = PathBuf::from(text);
    let path = if path.is_absolute() {
        path
    } else {
        root.join(path)
    };
    path.canonicalize().map_err(|e| e.to_string())
}
#[derive(Clone)]
struct Repository {
    workspace: PathBuf,
    root: PathBuf,
    admin: PathBuf,
    prefix: String,
    id: String,
}
fn repository(root: &Path) -> Result<Repository, String> {
    let workspace = root.canonicalize().map_err(|e| e.to_string())?;
    let repo = git_path(&workspace, &["rev-parse", "--show-toplevel"])?;
    let admin = git_path(&workspace, &["rev-parse", "--absolute-git-dir"])?;
    let relative = workspace
        .strip_prefix(&repo)
        .map_err(|_| "Workspace is outside its repository")?;
    let prefix = relative
        .to_str()
        .ok_or("Workspace repository prefix must be UTF-8")?
        .replace(std::path::MAIN_SEPARATOR, "/");
    let prefix = if prefix.is_empty() {
        prefix
    } else {
        prefix + "/"
    };
    #[cfg(unix)]
    let identity = {
        use std::os::unix::fs::MetadataExt;
        let m = std::fs::metadata(&admin).map_err(|e| e.to_string())?;
        json!([repo, admin, m.dev(), m.ino()])
    };
    #[cfg(not(unix))]
    let identity = json!([repo, admin]);
    let id = hash(&serde_json::to_vec(&identity).map_err(|e| e.to_string())?);
    Ok(Repository {
        workspace,
        root: repo,
        admin,
        prefix,
        id,
    })
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub path: String,
    pub mode: String,
    pub oid: String,
    pub stage: u8,
}
fn entries(bytes: &[u8]) -> Result<Vec<Entry>, String> {
    let mut out = Vec::new();
    for row in bytes.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        let tab = row
            .iter()
            .position(|b| *b == b'\t')
            .ok_or("Malformed Git index record")?;
        let header = std::str::from_utf8(&row[..tab]).map_err(|_| "Malformed Git index header")?;
        let fields = header.split(' ').collect::<Vec<_>>();
        if fields.len() != 3 {
            return Err("Malformed Git index stage".into());
        }
        let stage = fields[2].parse::<u8>().map_err(|_| "Malformed Git stage")?;
        if stage > 3
            || !matches!(fields[1].len(), 40 | 64)
            || !fields[1].bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("Malformed Git index identity".into());
        }
        out.push(Entry {
            path: String::from_utf8(row[tab + 1..].to_vec())
                .map_err(|_| "Non-UTF-8 conflict paths are unavailable in this provider")?,
            mode: fields[0].into(),
            oid: fields[1].into(),
            stage,
        });
    }
    Ok(out)
}
fn read_index(repo: &Repository) -> Result<Vec<u8>, String> {
    let path = repo.admin.join("index");
    let meta = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > MAX_INDEX as u64 {
        return Err("Git index must be a bounded regular file, not an alias.".into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(MAX_INDEX as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > MAX_INDEX {
        return Err("Git index exceeds its interactive budget".into());
    }
    Ok(bytes)
}
fn heads(repo: &Repository) -> Result<BTreeMap<String, Option<String>>, String> {
    let mut values = BTreeMap::new();
    for name in ["HEAD", "MERGE_HEAD", "CHERRY_PICK_HEAD", "REBASE_HEAD"] {
        let mut c = command(&repo.root);
        let o = c
            .args(["rev-parse", "--verify", "-q", name])
            .output()
            .map_err(|e| e.to_string())?;
        values.insert(
            name.into(),
            o.status
                .success()
                .then(|| String::from_utf8_lossy(&o.stdout).trim().into()),
        );
    }
    Ok(values)
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub version: u32,
    pub repository: String,
    pub fingerprint: String,
    pub index_hash: String,
    pub heads: BTreeMap<String, Option<String>>,
    pub entries: Vec<Entry>,
    pub outside_conflicts: usize,
}
fn observe(repo: &Repository) -> Result<Observation, String> {
    let raw = read_index(repo)?;
    let logical = git(&repo.root, &["ls-files", "--stage", "-z"], None, None)?;
    let all = entries(&logical)?;
    let mut selected = Vec::new();
    let mut outside = 0;
    for mut e in all.into_iter().filter(|e| e.stage > 0) {
        if let Some(path) = e.path.strip_prefix(&repo.prefix) {
            e.path = path.into();
            selected.push(e)
        } else {
            outside += 1;
        }
    }
    if selected.len() > 3000 {
        return Err("Conflict group overview exceeds 1000 workspace paths. Narrow the workspace or inspect Git externally.".into());
    }
    let heads = heads(repo)?;
    let index_hash = hash(&raw);
    let fingerprint = hash(
        &serde_json::to_vec(&json!([repo.id, index_hash, hash(&logical), heads]))
            .map_err(|e| e.to_string())?,
    );
    if read_index(repo)? != raw {
        return Err("Git index changed during observation. Read it again.".into());
    }
    Ok(Observation {
        version: 1,
        repository: repo.id.clone(),
        fingerprint,
        index_hash,
        heads,
        entries: selected,
        outside_conflicts: outside,
    })
}
fn same_observation(repo: &Repository, expected: &Observation) -> Result<(), String> {
    let fresh = observe(repo)?;
    if &fresh != expected {
        return Err("Git index, merge context or conflict group changed after review. Read conflicts again.".into());
    }
    Ok(())
}
struct IndexLease {
    repo: Repository,
    expected: Observation,
    file: std::fs::File,
    path: PathBuf,
    handle: same_file::Handle,
}
impl IndexLease {
    fn acquire(repo: Repository, expected: &Observation) -> Result<Self, String> {
        #[cfg(not(unix))]return Err("This provider cannot yet prove a stable repository index identity on this engine platform.".into());
        let path = repo.admin.join("index.lock");
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| format!("Git index is busy or unavailable; no source was changed: {e}"))?;
        let handle = same_file::Handle::from_file(file.try_clone().map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        let lease = Self {
            repo,
            expected: expected.clone(),
            file,
            path,
            handle,
        };
        same_observation(&lease.repo, expected)?;
        Ok(lease)
    }
    fn check(&self) -> Result<(), String> {
        let current = repository(&self.repo.workspace)?;
        if current.id != self.repo.id
            || same_file::Handle::from_path(&self.path).map_err(|e| e.to_string())? != self.handle
            || hash(&read_index(&self.repo)?) != self.expected.index_hash
            || heads(&self.repo)? != self.expected.heads
        {
            return Err("Git index lease or merge context changed. No further source/index changes are authorized.".into());
        }
        Ok(())
    }
}
impl Drop for IndexLease {
    fn drop(&mut self) {
        if same_file::Handle::from_path(&self.path).ok().as_ref() == Some(&self.handle) {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

/// Legacy per-path staging cannot be an alternate, partial conflict resolver.
pub(super) fn legacy_unmerged(root: &Path, paths: &[String], all: bool) -> Result<bool, String> {
    let repo = repository(root)?;
    let every = git(
        &repo.root,
        &["ls-files", "--unmerged", "--stage", "-z"],
        None,
        None,
    )?;
    if every.is_empty() {
        return Ok(false);
    }
    if all {
        return Ok(true);
    }
    let mut args = vec!["ls-files", "--unmerged", "--stage", "-z", "--"];
    args.extend(paths.iter().map(String::as_str));
    if !git(root, &args, None, None)?.is_empty() {
        return Ok(true);
    }
    let indexed = entries(&git(
        &repo.root,
        &["ls-files", "--stage", "-z"],
        None,
        None,
    )?)?
    .into_iter()
    .map(|e| e.path)
    .collect::<BTreeSet<_>>();
    let unmerged = entries(&every)?;
    for path in paths {
        if path
            .split('/')
            .any(|p| p.is_empty() || p == "." || p == "..")
            || Path::new(path).is_absolute()
        {
            return Err("Use exact listed Git paths; aliased path spellings cannot bypass conflict group review.".into());
        }
        let full = format!("{}{}", repo.prefix, path);
        if !indexed.contains(&full)
            && unmerged.iter().any(|e| {
                same_file::is_same_file(root.join(path), repo.root.join(&e.path)).unwrap_or(false)
            })
        {
            return Ok(true);
        }
    }
    Ok(false)
}
</ui:copy>

## Derive the complete source namespace from explicit choices

The caller selects group members and every retained destination. Stage references include the exact object identity, not an inferred branch direction. Omitted members are explicitly deleted; retained editor identities are chosen once from existing group sources, while new identities are explicit. A destination cannot overwrite an unrelated existing source. The resource planner validates the resulting namespace and the shared acquisition/candidate mechanisms remain authoritative for filesystem spelling, permissions, dirty drafts, construction ownership and generated consequences. Saved/draft states and executable-mode choices are retained separately.

No source or index is changed during preparation. Incompatible physical-source spellings remain readable but unavailable for publication; the shared codec is not widened here. Staging exact saved bytes is unavailable when Git attributes would run a clean filter, encoding/ident conversion or unpreviewed newline transformation. There is no raw-byte fallback that silently ignores such configuration.

<ui:copy id="git-resolution-plan">
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadRequest {
    pub version: u32,
    #[serde(default)]
    pub members: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageText {
    pub stage: u8,
    pub mode: String,
    pub oid: String,
    pub text: Option<String>,
    pub unavailable: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathReading {
    pub path: String,
    pub stages: Vec<StageText>,
    pub working: Option<SavedSource>,
    pub unavailable: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reading {
    pub version: u32,
    pub resolution_protocol: u32,
    pub observation: Observation,
    pub members: Vec<String>,
    pub paths: Vec<PathReading>,
    pub index_modified: bool,
    pub source_files_modified: bool,
}
fn selected_members(
    observation: &Observation,
    members: &[String],
) -> Result<BTreeSet<String>, String> {
    let selected: BTreeSet<_> = members.iter().cloned().collect();
    if selected.len() != members.len()
        || selected.len() > 1000
        || !selected
            .iter()
            .all(|path| observation.entries.iter().any(|entry| &entry.path == path))
    {
        return Err("Select unique current unmerged workspace paths as one explicit group.".into());
    }
    Ok(selected)
}
fn reading(root: &Path, request: &ReadRequest) -> Result<Reading, String> {
    if request.version != 1 {
        return Err("Unsupported conflict resolution protocol".into());
    }
    let repo = repository(root)?;
    let observation = observe(&repo)?;
    let selected = selected_members(&observation, &request.members)?;
    let paths: BTreeSet<_> = observation.entries.iter().map(|e| e.path.clone()).collect();
    let mut result = Vec::new();
    let mut bytes = 0usize;
    for path in paths {
        let mut unavailable = super::resource_changes::validate_path(&path).err();
        let working = if selected.contains(&path) && unavailable.is_none() {
            match super::resource_inputs::saved_target(root, &path) {
                Ok((_, saved)) => {
                    bytes = bytes.saturating_add(
                        serde_json::to_vec(&saved).map_err(|e| e.to_string())?.len(),
                    );
                    Some(saved)
                }
                Err(error) => {
                    unavailable = Some(error);
                    None
                }
            }
        } else {
            None
        };
        let mut stages = Vec::new();
        for entry in observation.entries.iter().filter(|e| e.path == path) {
            let mut stage = StageText {
                stage: entry.stage,
                mode: entry.mode.clone(),
                oid: entry.oid.clone(),
                text: None,
                unavailable: None,
            };
            if !selected.contains(&path) {
                stage.unavailable =
                    Some("Select this path to inspect its retained text stages.".into());
            } else if !matches!(entry.mode.as_str(), "100644" | "100755") {
                stage.unavailable = Some("This stage is not a regular text file.".into());
            } else {
                let size = git(&repo.root, &["cat-file", "-s", &entry.oid], None, None)?;
                let size = std::str::from_utf8(&size)
                    .map_err(|_| "Invalid Git object size")?
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| "Invalid Git object size")?;
                if size > 2 * 1024 * 1024 || bytes.saturating_add(size) > 16 * 1024 * 1024 {
                    stage.unavailable =
                        Some("Stage exceeds the bounded complete text reading budget.".into());
                } else {
                    let blob = git(&repo.root, &["cat-file", "blob", &entry.oid], None, None)?;
                    if blob.len() != size {
                        return Err("Git stage changed while acquiring text.".into());
                    }
                    match String::from_utf8(blob) {
                        Ok(text) if !text.contains('\0') => {
                            bytes += text.len();
                            stage.text = Some(text)
                        }
                        _ => {
                            stage.unavailable =
                                Some("Binary or non-UTF-8 stage text is unavailable.".into())
                        }
                    }
                }
            }
            stages.push(stage);
        }
        if bytes > 16 * 1024 * 1024 {
            return Err("Selected conflict reading exceeds 16 MiB; select a smaller group.".into());
        }
        result.push(PathReading {
            path,
            stages,
            working,
            unavailable,
        });
    }
    same_observation(&repo, &observation)?;
    Ok(Reading {
        version: 1,
        resolution_protocol: 1,
        observation,
        members: selected.into_iter().collect(),
        paths: result,
        index_modified: false,
        source_files_modified: false,
    })
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Content {
    Text {
        text: String,
    },
    Stage {
        path: String,
        stage: u8,
        oid: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Destination {
    pub path: String,
    pub identity_from: Option<String>,
    pub content: Content,
    pub mode: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrepareRequest {
    pub version: u32,
    pub reading: Reading,
    pub members: Vec<String>,
    pub outputs: Vec<Destination>,
    pub inputs: Inputs,
    pub capture: super::resource_inputs::Request,
}
fn targets(request: &PrepareRequest) -> Result<BTreeSet<String>, String> {
    let members = selected_members(&request.reading.observation, &request.members)?;
    if members.is_empty() || request.outputs.len() > 1000 {
        return Err("Choose a bounded nonempty complete conflict group.".into());
    }
    let mut paths = members.clone();
    let mut destinations = BTreeSet::new();
    let mut identities = BTreeSet::new();
    for output in &request.outputs {
        super::resource_changes::validate_path(&output.path)?;
        if !destinations.insert(output.path.clone())
            || !matches!(output.mode.as_str(), "100644" | "100755")
        {
            return Err("Resolved destinations must be unique regular file paths with an explicit Git mode.".into());
        }
        if let Some(from) = &output.identity_from {
            if !members.contains(from) || !identities.insert(from.clone()) {
                return Err(
                    "Choose each retained source identity explicitly once from this group.".into(),
                );
            }
        }
        if let Content::Stage { path, stage, .. } = &output.content {
            if !members.contains(path) || !matches!(stage, 1..=3) {
                return Err("Choose an existing stage from this complete group.".into());
            }
        }
        paths.insert(output.path.clone());
    }
    for path in &paths {
        super::resource_changes::validate_path(path)?;
    }
    Ok(paths)
}
fn resolve_text(output: &Destination, reading: &Reading) -> Result<String, String> {
    let text = match &output.content {
        Content::Text { text } => text.clone(),
        Content::Stage { path, stage, oid } => {
            let selected = reading
                .paths
                .iter()
                .find(|p| &p.path == path)
                .and_then(|p| p.stages.iter().find(|s| s.stage == *stage && &s.oid == oid))
                .ok_or("Selected stage identity changed; inspect the group again")?;
            selected.text.clone().ok_or_else(|| {
                selected
                    .unavailable
                    .clone()
                    .unwrap_or("Stage text unavailable".into())
            })?
        }
    };
    if text.len() > 8 * 1024 * 1024 || text.contains('\0') {
        return Err("Resolved source must be bounded UTF-8 text without NUL.".into());
    }
    Ok(text)
}
fn resource_plan(request: &PrepareRequest, fresh: &Reading) -> Result<ResourcePlan, String> {
    let paths = targets(request)?;
    if request
        .inputs
        .sources
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>()
        != paths
    {
        return Err(
            "Acquire the complete exact source/destination namespace before reviewing the group."
                .into(),
        );
    }
    let mut after: BTreeMap<String, SourceState> = request
        .inputs
        .sources
        .keys()
        .map(|p| (p.clone(), SourceState::Missing { creatable: true }))
        .collect();
    let members: BTreeSet<_> = request.members.iter().cloned().collect();
    for output in &request.outputs {
        if !members.contains(&output.path)
            && matches!(
                request.inputs.sources.get(&output.path),
                Some(SourceState::File { .. })
            )
        {
            return Err("A resolution destination already owns an unrelated source. Select it explicitly as a group member or choose confirmed absence.".into());
        }
        let (identity, version) = if let Some(from) = &output.identity_from {
            match request.inputs.sources.get(from){Some(SourceState::File{identity,version,..})=>(identity.clone(),*version),_=>return Err("The selected retained editor source is missing. Choose a new identity explicitly.".into())}
        } else {
            (
                format!(
                    "git-resolution:{}",
                    hash(format!("{}:{}", request.inputs.context, output.path).as_bytes())
                ),
                None,
            )
        };
        after.insert(
            output.path.clone(),
            SourceState::File {
                identity,
                text: resolve_text(output, fresh)?,
                writable: true,
                removable: true,
                version,
            },
        );
    }
    let patches = paths
        .into_iter()
        .filter_map(|path| {
            let before = request.inputs.sources[&path].clone();
            let desired = after[&path].clone();
            if matches!(
                (&before, &desired),
                (SourceState::Missing { .. }, SourceState::Missing { .. })
            ) {
                None
            } else {
                Some(ResourcePatch {
                    path,
                    before,
                    after: desired,
                })
            }
        })
        .collect();
    let plan = ResourcePlan {
        version: 1,
        context: request.inputs.context.clone(),
        title: "Resolve selected Git conflict group".into(),
        patches,
        annotations: Vec::new(),
    };
    plan.apply(&plan.context, &request.inputs.sources)?;
    Ok(plan)
}
fn apply_modes(manifest: &mut super::resource_journal::Manifest, outputs: &[Destination]) {
    for output in outputs {
        if let Some(SavedSource::File {
            readonly,
            executable,
            mode,
            ..
        }) = manifest.after.get_mut(&output.path)
        {
            *readonly = false;
            *executable = output.mode == "100755";
            #[cfg(unix)]
            {
                *mode = Some(mode.unwrap_or(0o644) & !0o111 | if *executable { 0o111 } else { 0 });
            }
        }
    }
}
fn current_saved(root: &Path, expected: &BTreeMap<String, SavedSource>) -> Result<(), String> {
    for (path, state) in expected {
        if &super::resource_inputs::saved_target(root, path)?.1 != state {
            return Err(format!(
                "Saved source changed after conflict review: {path}. No index changes were made."
            ));
        }
    }
    Ok(())
}
fn stage_policy(
    repo: &Repository,
    outputs: &[Destination],
    saved: &BTreeMap<String, SavedSource>,
) -> Result<(), String> {
    if outputs.is_empty() {
        return Ok(());
    }
    let mut input = Vec::new();
    for output in outputs {
        input.extend_from_slice(format!("{}{}", repo.prefix, output.path).as_bytes());
        input.push(0);
    }
    let attrs = git(
        &repo.root,
        &[
            "check-attr",
            "-z",
            "--stdin",
            "filter",
            "working-tree-encoding",
            "ident",
            "text",
            "eol",
        ],
        Some(&input),
        None,
    )?;
    let rows = attrs
        .split(|b| *b == 0)
        .filter(|r| !r.is_empty())
        .collect::<Vec<_>>();
    if rows.len() % 3 != 0 {
        return Err("Git attributes could not be read completely.".into());
    }
    let mut values: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    for row in rows.chunks(3) {
        let path = std::str::from_utf8(row[0]).map_err(|_| "Invalid Git attribute path")?;
        let key = std::str::from_utf8(row[1]).map_err(|_| "Invalid Git attribute name")?;
        let value = std::str::from_utf8(row[2]).map_err(|_| "Invalid Git attribute value")?;
        values
            .entry(path.into())
            .or_default()
            .insert(key.into(), value.into());
    }
    let autocrlf = command(&repo.root)
        .args(["config", "--get", "core.autocrlf"])
        .output()
        .map_err(|e| e.to_string())?;
    let autocrlf = String::from_utf8_lossy(&autocrlf.stdout)
        .trim()
        .to_ascii_lowercase();
    for output in outputs {
        let key = format!("{}{}", repo.prefix, output.path);
        let attrs = values
            .get(&key)
            .ok_or("Git attribute reading is incomplete")?;
        let active = |name: &str| {
            attrs
                .get(name)
                .is_some_and(|v| v != "unspecified" && v != "unset")
        };
        if ["filter", "working-tree-encoding", "ident"]
            .iter()
            .any(|k| active(k))
        {
            return Err(format!(
                "Staging {0} requires a Git filter, encoding or ident transformation whose resulting blob is not previewed by this provider. Source resolution remains available; no raw-byte fallback will stage it.",
                output.path
            ));
        }
        if let Some(SavedSource::File { text, .. }) = saved.get(&output.path) {
            if text.contains("\r\n")
                && (active("text")
                    || active("eol")
                    || matches!(autocrlf.as_str(), "true" | "input"))
            {
                return Err("Git newline conversion requires a transformed-blob review, unavailable in this provider. No index changes were made.".into());
            }
        }
    }
    Ok(())
}
</ui:copy>
## Retain review authority independently of renderer state

A bounded engine-held review stores the exact observed group, resource proposal, generated consequences and saved states. Clients act with its random token and exact fingerprint; a payload cannot substitute new text or destinations after review. The existing root-scoped save-attempt authority binds each one-shot request identity and retains status after a lost HTTP reply. An evicted review requires a fresh inspection, never automatic replay.

Draft application revalidates the index and source consequences but changes no saved file. Explicit Save uses the existing live-room/resource publisher, additionally holding the real index lease; its conflict-only rollback policy may restore only proven unchanged writes. An uncertain or incomplete save never authorizes staging or editor adoption, and ordinary partial-publication semantics are unchanged. This is recoverable multi-file publication, not cross-file crash atomicity.

Stage is a separate explicit operation, permitted only after a complete source confirmation or when saved bytes already equal the reviewed result. It re-reads current source permission/ownership and construction consequences, rechecks after preparing a temporary index, and installs the entire selected group atomically. Other unmerged paths remain explicitly listed; this operation does not claim the repository is conflict-free or that the composed text is semantically correct.

<ui:copy id="git-resolution-service">
#[derive(Serialize)]
struct Prepared {
    request: PrepareRequest,
    proposal: super::resource_candidates::Request,
    manifest: super::resource_journal::Manifest,
    facts: Value,
    full_after: BTreeMap<String, SavedSource>,
    fingerprint: String,
    needs_save: bool,
    stage_unavailable: Option<String>,
    #[serde(skip)]
    published_guards:Mutex<Option<BTreeMap<String,String>>>,
}
struct Record {
    plan: Arc<Prepared>,
    cost: usize,
    active: bool,
    save_attempted: bool,
    saved: bool,
    stage_attempted: bool,
}
#[derive(Default)]
struct Reviews {
    records: BTreeMap<String, Record>,
    order: std::collections::VecDeque<String>,
    bytes: usize,
}
static REVIEWS: std::sync::LazyLock<Mutex<Reviews>> =
    std::sync::LazyLock::new(|| Mutex::new(Reviews::default()));
fn hold(plan: Prepared) -> Result<String, String> {
    let cost = serde_json::to_vec(&plan)
        .map_err(|e| e.to_string())?
        .len()
        .checked_mul(4)
        .and_then(|n| n.checked_add(65536))
        .ok_or("Conflict review size overflow")?;
    let mut cache = REVIEWS
        .lock()
        .map_err(|_| "Conflict review cache is unavailable")?;
    if cost > 128 * 1024 * 1024 {
        return Err("Conflict review exceeds its retained interactive budget; choose a smaller complete group.".into());
    }
    while cache.records.len() >= 32 || cache.bytes.saturating_add(cost) > 128 * 1024 * 1024 {
        let old=cache.order.iter().position(|id|cache.records.get(id).is_some_and(|r|!r.active)).ok_or("Conflict review capacity is held by pending publications. Inspect their status first")?;
        let id = cache.order.remove(old).unwrap();
        if let Some(record) = cache.records.remove(&id) {
            cache.bytes -= record.cost;
        }
    }
    let token = nonce()?;
    cache.bytes += cost;
    cache.order.push_back(token.clone());
    cache.records.insert(
        token.clone(),
        Record {
            plan: Arc::new(plan),
            cost,
            active: false,
            save_attempted: false,
            saved: false,
            stage_attempted: false,
        },
    );
    Ok(token)
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Act {
    pub version: u32,
    pub token: String,
    pub fingerprint: String,
    pub request_id: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Check {
    pub version: u32,
    pub token: String,
    pub fingerprint: String,
    #[serde(default)]
    pub stage: bool,
}
fn held(state: &LocalState, token: &str, fingerprint: &str) -> Result<Arc<Prepared>, String> {
    let cache = REVIEWS
        .lock()
        .map_err(|_| "Conflict review cache unavailable")?;
    let record=cache.records.get(token).ok_or("Conflict review expired. Inspect saved files/recovery and prepare a fresh group; no write was retried")?;
    if record.plan.fingerprint != fingerprint
        || state.resource_root.check(state.index.root())? != record.plan.proposal.inputs.workspace
    {
        return Err("Conflict review fingerprint or workspace identity changed.".into());
    }
    Ok(record.plan.clone())
}
fn begin_act(token: &str, stage: bool) -> Result<(), String> {
    let mut cache = REVIEWS
        .lock()
        .map_err(|_| "Conflict review cache unavailable")?;
    let r = cache
        .records
        .get_mut(token)
        .ok_or("Conflict review expired")?;
    if r.active
        || if stage {
            r.stage_attempted || r.plan.needs_save && !r.saved
        } else {
            r.save_attempted || r.stage_attempted
        }
    {
        return Err("This review is pending, already consumed, or its complete source save is unconfirmed. Inspect status; prepare a fresh review rather than retrying.".into());
    }
    r.active = true;
    if stage {
        r.stage_attempted = true
    } else {
        r.save_attempted = true
    };
    Ok(())
}
fn finish_act(token: &str, saved: bool) {
    if let Ok(mut cache) = REVIEWS.lock() {
        if let Some(r) = cache.records.get_mut(token) {
            r.active = false;
            r.saved |= saved;
        }
    }
}
fn response(token: &str, p: &Prepared) -> Value {
    json!({"version":1,"resolution_protocol":1,"token":token,"fingerprint":p.fingerprint,"members":p.request.members,"remaining_conflicts":p.request.reading.paths.iter().filter(|v|!p.request.members.contains(&v.path)).map(|v|v.path.clone()).collect::<Vec<_>>(),"proposal":p.proposal,"plan":p.proposal.plan,"consequences":p.facts,"saved_sources":p.manifest.after,"needs_save":p.needs_save,"stage_available":p.stage_unavailable.is_none(),"stage_unavailable":p.stage_unavailable,"source_files_modified":false,"index_modified":false})
}
pub async fn read(
    State(state): State<LocalState>,
    Json(request): Json<ReadRequest>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    tokio::task::spawn_blocking(move || {
        reading(&root, &request).and_then(|v| serde_json::to_value(v).map_err(|e| e.to_string()))
    })
    .await
    .map_err(|e| ApiError::internal(e.to_string()))?
    .map(Json)
    .map_err(ApiError::conflict)
}
pub async fn prepare(
    State(state): State<LocalState>,
    Json(request): Json<PrepareRequest>,
) -> ApiResult<Json<Value>> {
    if request.version != 1 {
        return Err(ApiError::bad_request(
            "Unsupported conflict preparation protocol",
        ));
    }
    let root = state.index.root().to_path_buf();
    let read_request = ReadRequest {
        version: 1,
        members: request.members.clone(),
    };
    let fresh = tokio::task::spawn_blocking(move || reading(&root, &read_request))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .map_err(ApiError::conflict)?;
    if serde_json::to_value(&fresh).map_err(|e| ApiError::internal(e.to_string()))?
        != serde_json::to_value(&request.reading).map_err(|e| ApiError::internal(e.to_string()))?
    {
        return Err(ApiError::conflict(
            "Conflict stages or working sources changed since inspection. Read the selected group again; composed text may be retained separately.",
        ));
    }
    let paths = targets(&request).map_err(ApiError::bad_request)?;
    let captured = super::resource_changes::targets(&request.capture.edit)
        .map_err(ApiError::bad_request)?
        .into_iter()
        .collect::<BTreeSet<_>>();
    if paths != captured {
        return Err(ApiError::conflict(
            "Resource capture does not cover exactly the complete selected group and destinations.",
        ));
    }
    let plan = resource_plan(&request, &fresh).map_err(ApiError::conflict)?;
    let proposal = super::resource_candidates::Request {
        inputs: request.inputs.clone(),
        capture: request.capture.clone(),
        plan,
    };
    let mut facts =
        super::resource_candidates::inspect(State(state.clone()), Json(proposal.clone()))
            .await?
            .0;
    if !facts.protected_targets.is_empty() {
        return Err(ApiError::conflict(
            "A resolution target becomes generated; resolve its authored construction instead.",
        ));
    }
    let mut manifest =
        super::resource_journal::Manifest::from_review(&proposal.inputs, &proposal.plan)
            .map_err(ApiError::conflict)?;
    apply_modes(&mut manifest, &request.outputs);
    facts.saved_sources = manifest.after.clone();
    let mut full_after = proposal.inputs.saved_sources.clone();
    full_after.extend(manifest.after.clone());
    let needs_save = manifest.before != manifest.after
        || proposal.plan.patches.iter().any(|p| p.before != p.after);
    let repo = repository(state.index.root()).map_err(ApiError::conflict)?;
    same_observation(&repo, &fresh.observation).map_err(ApiError::conflict)?;
    let stage_unavailable = stage_policy(&repo, &request.outputs, &full_after).err();
    let facts = serde_json::to_value(facts).map_err(|e| ApiError::internal(e.to_string()))?;
    let fingerprint = hash(
        &serde_json::to_vec(&json!([
            request,
            proposal,
            manifest,
            facts,
            full_after,
            needs_save,
            stage_unavailable
        ]))
        .map_err(|e| ApiError::internal(e.to_string()))?,
    );
    let token = hold(Prepared {
        request,
        proposal,
        manifest,
        facts,
        full_after,
        fingerprint,
        needs_save,
        stage_unavailable,
        published_guards:Mutex::new(None),
    })
    .map_err(ApiError::conflict)?;
    let p = held(&state, &token, &{
        let c = REVIEWS
            .lock()
            .map_err(|_| ApiError::internal("Conflict cache unavailable"))?;
        c.records[&token].plan.fingerprint.clone()
    })
    .map_err(ApiError::conflict)?;
    Ok(Json(response(&token, &p)))
}
async fn validate_plan(state: &LocalState, p: &Prepared) -> Result<(), String> {
    let repo = repository(state.index.root())?;
    same_observation(&repo, &p.request.reading.observation)?;
    let facts = super::resource_candidates::inspect(State(state.clone()), Json(p.proposal.clone()))
        .await
        .map_err(|e| e.detail().to_string())?
        .0;
    if facts.fingerprint
        != p.facts["fingerprint"]
            .as_str()
            .ok_or("Incomplete source consequences")?
    {
        return Err("Source/generated consequences changed after review.".into());
    }
    Ok(())
}
pub async fn validate(
    State(state): State<LocalState>,
    Json(request): Json<Check>,
) -> ApiResult<Json<Value>> {
    if request.version != 1 {
        return Err(ApiError::bad_request(
            "Unsupported conflict validation protocol",
        ));
    }
    let p = held(&state, &request.token, &request.fingerprint).map_err(ApiError::conflict)?;
    if request.stage {
        let repo=repository(state.index.root()).map_err(ApiError::conflict)?;
        let lease=IndexLease::acquire(repo,&p.request.reading.observation).map_err(ApiError::conflict)?;
        current_saved(state.index.root(),&p.full_after).map_err(ApiError::conflict)?;
        stage_authority(&state,&p).await.map_err(ApiError::conflict)?;
        lease.check().map_err(ApiError::conflict)?;
        current_saved(state.index.root(),&p.full_after).map_err(ApiError::conflict)?;
    } else {
        validate_plan(&state, &p).await.map_err(ApiError::conflict)?;
    }
    Ok(Json(
        json!({"version":1,"token":request.token,"fingerprint":request.fingerprint,"stage":request.stage,"current":true,"plan":p.proposal.plan,"saved_sources":p.manifest.after,"source_files_modified":false,"index_modified":false}),
    ))
}
pub async fn save(
    State(state): State<LocalState>,
    Json(request): Json<Act>,
) -> ApiResult<Json<Value>> {
    state
        .resource_root
        .ready()
        .await
        .map_err(ApiError::unavailable)?;
    let workspace = state
        .resource_root
        .check(state.index.root())
        .map_err(ApiError::conflict)?
        .to_string();
    if request.version != 1 {
        return Err(ApiError::bad_request("Unsupported conflict save protocol"));
    }
    if let Some(result) = super::resource_saves::reserve(
        &workspace,
        &request.request_id,
        &request.fingerprint,
        &("git_conflict_save", &request),
    )
    .map_err(ApiError::conflict)?
    {
        return match result{Ok(value)=>super::resource_saves::checked_retained(&state,value).await.map(Json).map_err(ApiError::conflict),Err(error)=>Err(ApiError::conflict(error))};
    }
    let result=async{let p=held(&state,&request.token,&request.fingerprint)?;if !p.needs_save{return Err("This group already has its reviewed saved bytes; choose explicit staging instead of a no-op save.".into());}begin_act(&request.token,false)?;validate_plan(&state,&p).await?;let repo=repository(state.index.root())?;let lease=IndexLease::acquire(repo,&p.request.reading.observation)?;let mut facts=super::resource_candidates::inspect(State(state.clone()),Json(p.proposal.clone())).await.map_err(|e|e.detail().to_string())?.0;facts.saved_sources=p.manifest.after.clone();
 let value=super::resource_saves::publish_guarded(state.clone(),p.manifest.clone(),facts,request.request_id.clone(),p.proposal.inputs.saved_guards.clone(),p.proposal.inputs.ownership.protected_entries.clone(),super::resource_saves::EntryTransition::Stable,Box::new(move |_|lease.check()),true).await?;Ok(value)}.await;
    let confirmed = result
        .as_ref()
        .ok()
        .is_some_and(|v| v["version"]==2 && v["outcome"]["complete"] == true && v["adoption_current"]==true);
    if confirmed{if let Ok(p)=held(&state,&request.token,&request.fingerprint){let guards=result.as_ref().ok().and_then(|value|value["source_files"].as_array()).map(|rows|rows.iter().filter_map(|row|Some((row["file"]["path"].as_str()?.to_owned(),row["save_guard"].as_str()?.to_owned()))).collect());if let Ok(mut published)=p.published_guards.lock(){*published=guards};}}
    finish_act(&request.token, confirmed);
    super::resource_saves::retain(&workspace, &request.request_id, &result);
    result.map(Json).map_err(ApiError::conflict)
}
async fn stage_authority(state: &LocalState, p: &Prepared) -> Result<(), String> {
    let mut capture = p.proposal.capture.clone();
    capture.drafts.clear();capture.saved_guards=p.proposal.inputs.saved_guards.clone();
    let after_guards=p.published_guards.lock().map_err(|_|"Saved source proof is unavailable")?.clone();
    if p.needs_save && after_guards.is_none(){return Err("Source save has no retained producer-issued physical guards. Inspect its publication before staging.".into())}
    for (path, saved) in &p.full_after {
            match saved {
                SavedSource::File { text, .. } => {
                    capture.ownership_sources.insert(path.clone(), text.clone());
                }
                SavedSource::Missing => {
                    capture.ownership_sources.remove(path);
                }
            }
        match saved{SavedSource::File{..}=>{if let Some(guards)=&after_guards{let guard=guards.get(path).ok_or_else(||format!("Saved source has no producer guard: {path}"))?;capture.saved_guards.insert(path.clone(),guard.clone());}},SavedSource::Missing=>{capture.saved_guards.remove(path);}}
    }
    let inputs = super::resource_inputs::inspect(State(state.clone()), Json(capture.clone()))
        .await
        .map_err(|e| e.detail().to_string())?
        .0;
    if inputs.saved_sources != p.full_after || !inputs.refusals.is_empty() {
        return Err(
            "Saved source namespace, permission or ownership changed after resolution review."
                .into(),
        );
    }
    let plan = ResourcePlan {
        version: 1,
        context: inputs.context.clone(),
        title: "Validate saved resolution staging".into(),
        patches: Vec::new(),
        annotations: Vec::new(),
    };
    let facts = super::resource_candidates::inspect(
        State(state.clone()),
        Json(super::resource_candidates::Request {
            inputs,
            capture,
            plan,
        }),
    )
    .await
    .map_err(|e| e.detail().to_string())?
    .0;
    let fresh = serde_json::to_value(facts).map_err(|e| e.to_string())?;
    for layer in ["saved", "draft"] {
        for key in ["entries", "source_hashes", "output_paths"] {
            if fresh[layer][key] != p.facts[layer][key] {
                return Err("Construction/source/generated consequences changed before staging. Read and review the group again.".into());
            }
        }
    }
    Ok(())
}
fn atomic_stage(
    root: &Path,
    p: &Prepared,
    mut authority: impl FnMut() -> Result<(), String>,
) -> Result<Value, String> {
    let repo = repository(root)?;
    let mut lease = IndexLease::acquire(repo, &p.request.reading.observation)?;
    authority()?;
    current_saved(root, &p.full_after)?;
    stage_policy(&lease.repo, &p.request.outputs, &p.full_after)?;
    let original = read_index(&lease.repo)?;
    let mut temporary =
        tempfile::NamedTempFile::new_in(&lease.repo.admin).map_err(|e| e.to_string())?;
    temporary.write_all(&original).map_err(|e| e.to_string())?;
    temporary.flush().map_err(|e| e.to_string())?;
    let mut update = Vec::new();
    let zero = "0".repeat(
        p.request
            .reading
            .observation
            .entries
            .first()
            .ok_or("Conflict group is empty")?
            .oid
            .len(),
    );
    let mut touched: BTreeSet<String> = p
        .request
        .members
        .iter()
        .map(|path| format!("{}{}", lease.repo.prefix, path))
        .collect();
    touched.extend(
        p.request
            .outputs
            .iter()
            .map(|o| format!("{}{}", lease.repo.prefix, o.path)),
    );
    for path in &touched {
        update.extend_from_slice(format!("0 {zero}\t{path}").as_bytes());
        update.push(0);
    }
    let mut resolved = BTreeMap::new();
    for output in &p.request.outputs {
        let SavedSource::File { text, .. } = &p.full_after[&output.path] else {
            return Err("Resolved output is absent".into());
        };
        let oid = String::from_utf8(git(
            &lease.repo.root,
            &["hash-object", "-w", "--stdin", "--no-filters"],
            Some(text.as_bytes()),
            None,
        )?)
        .map_err(|_| "Invalid Git blob identity")?;
        let oid = oid.trim().to_string();
        if !matches!(oid.len(), 40 | 64) || !oid.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("Git produced a malformed blob identity".into());
        }
        let path = format!("{}{}", lease.repo.prefix, output.path);
        update.extend_from_slice(format!("{} {}\t{}", output.mode, oid, path).as_bytes());
        update.push(0);
        resolved.insert(
            path,
            Entry {
                path: output.path.clone(),
                mode: output.mode.clone(),
                oid,
                stage: 0,
            },
        );
    }
    git(
        &lease.repo.root,
        &["update-index", "-z", "--index-info"],
        Some(&update),
        Some(temporary.path()),
    )?;
    let before = entries(&git(
        &lease.repo.root,
        &["ls-files", "--stage", "-z"],
        None,
        None,
    )?)?;
    let actual = entries(&git(
        &lease.repo.root,
        &["ls-files", "--stage", "-z"],
        None,
        Some(temporary.path()),
    )?)?;
    let mut expected: BTreeMap<_, _> = before
        .into_iter()
        .filter(|e| !touched.contains(&e.path))
        .map(|e| ((e.path.clone(), e.stage), e))
        .collect();
    for (path, mut e) in resolved {
        e.path = path.clone();
        expected.insert((path, 0), e);
    }
    let actual: BTreeMap<_, _> = actual
        .into_iter()
        .map(|e| ((e.path.clone(), e.stage), e))
        .collect();
    if actual != expected {
        return Err("Git did not prepare the entire exact resolution group. The real index was not changed.".into());
    }
    lease.check()?;
    authority()?;
    current_saved(root, &p.full_after)?;
    stage_policy(&lease.repo, &p.request.outputs, &p.full_after)?;
    let bytes = std::fs::read(temporary.path()).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_INDEX {
        return Err("Prepared Git index exceeds budget".into());
    }
    lease
        .file
        .set_permissions(
            std::fs::metadata(lease.repo.admin.join("index"))
                .map_err(|e| e.to_string())?
                .permissions(),
        )
        .map_err(|e| e.to_string())?;
    lease.file.write_all(&bytes).map_err(|e| e.to_string())?;
    lease.file.sync_all().map_err(|e| e.to_string())?;
    lease.check()?;
    std::fs::rename(&lease.path, lease.repo.admin.join("index")).map_err(|e| e.to_string())?;
    let synchronized = std::fs::File::open(&lease.repo.admin)
        .and_then(|f| f.sync_all())
        .is_ok();
    let remaining = expected
        .values()
        .filter(|e| e.stage > 0)
        .filter_map(|e| e.path.strip_prefix(&lease.repo.prefix).map(str::to_string))
        .collect::<BTreeSet<_>>();
    Ok(
        json!({"version":1,"operation":"git_conflict_stage","complete":synchronized,"index_modified":true,"index_uncertain":!synchronized,"source_files_modified":false,"resolved_members":p.request.members,"remaining_conflicts":remaining,"fingerprint":hash(&bytes)}),
    )
}
pub async fn stage(
    State(state): State<LocalState>,
    Json(request): Json<Act>,
) -> ApiResult<Json<Value>> {
    state
        .resource_root
        .ready()
        .await
        .map_err(ApiError::unavailable)?;
    let workspace = state
        .resource_root
        .check(state.index.root())
        .map_err(ApiError::conflict)?
        .to_string();
    if request.version != 1 {
        return Err(ApiError::bad_request("Unsupported conflict stage protocol"));
    }
    if let Some(result) = super::resource_saves::reserve(
        &workspace,
        &request.request_id,
        &request.fingerprint,
        &("git_conflict_stage", &request),
    )
    .map_err(ApiError::conflict)?
    {
        return match result{Ok(value)=>super::resource_saves::checked_retained(&state,value).await.map(Json).map_err(ApiError::conflict),Err(error)=>Err(ApiError::conflict(error))};
    }
    let mut result = async {
        let p = held(&state, &request.token, &request.fingerprint)?;
        if let Some(reason) = &p.stage_unavailable {
            return Err(reason.clone());
        }
        begin_act(&request.token, true)?;
        let root = state.index.root().to_path_buf();
        let runtime = tokio::runtime::Handle::current();
        let state = state.clone();
        tokio::task::spawn_blocking(move || {
            atomic_stage(&root, &p, || runtime.block_on(stage_authority(&state, &p)))
        })
        .await
        .map_err(|e| e.to_string())?
    }
    .await;
    if let Ok(value)=&mut result {
        value["request_id"]=json!(request.request_id);
        value["token"]=json!(request.token);
        value["review_fingerprint"]=json!(request.fingerprint);
    }
    finish_act(&request.token, false);
    super::resource_saves::retain(&workspace, &request.request_id, &result);
    result.map(Json).map_err(ApiError::conflict)
}
pub async fn status(
    State(state): State<LocalState>,
    Query(params): Query<super::resource_saves::Status>,
) -> ApiResult<Json<Value>> {
    super::resource_saves::retained_status(&state,&params.request_id).await.map(Json).map_err(ApiError::conflict)
}
</ui:copy>
## Exercise real Git index and source guards

The focused fixtures create actual textual, delete/modify and rename/rename merges. They verify composition, absent results, explicit retained identity/destination, immutable unrelated index entries, source/index/admission refusal and literal path handling. Unsupported namespace names remain readable without acquiring source-write authority. Service/controller acceptance, failure recovery and native editor retention are verified separately; these pure fixtures alone do not establish UI acceptance.

<ui:copy id="git-resolution-tests">
#[cfg(test)]
mod tests {
    use super::*;
    fn ok(root: &Path, args: &[&str]) {
        git(root, args, None, None).unwrap();
    }
    fn fixture(kind: &str, path: &str) -> tempfile::TempDir {
        let t = tempfile::tempdir().unwrap();
        let r = t.path();
        ok(r, &["init", "-q", "-b", "main"]);
        ok(r, &["config", "user.name", "Conflict fixture"]);
        ok(r, &["config", "user.email", "fixture@example.invalid"]);
        std::fs::write(r.join(path), "base\n").unwrap();
        std::fs::write(r.join("clean.py"), "clean\n").unwrap();
        ok(r, &["add", "--", path, "clean.py"]);
        ok(r, &["commit", "-qm", "base"]);
        ok(r, &["checkout", "-qb", "incoming"]);
        if kind == "rename" {
            ok(r, &["mv", "--", path, "right.py"]);
        } else {
            std::fs::write(r.join(path), "incoming\n").unwrap();
        }
        ok(r, &["commit", "-qam", "incoming"]);
        ok(r, &["checkout", "main"]);
        if kind == "rename" {
            ok(r, &["mv", "--", path, "left.py"]);
        } else if kind == "delete" {
            ok(r, &["rm", "--", path]);
        } else {
            std::fs::write(r.join(path), "current\n").unwrap();
        }
        ok(r, &["commit", "-qam", "current"]);
        assert!(git(r, &["merge", "incoming"], None, None).is_err());
        assert!(!observe(&repository(r).unwrap()).unwrap().entries.is_empty());
        t
    }
    fn prepared(root: &Path, outputs: Vec<Destination>) -> Prepared {
        let repo = repository(root).unwrap();
        let observation = observe(&repo).unwrap();
        let members = observation
            .entries
            .iter()
            .map(|e| e.path.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let read = reading(
            root,
            &ReadRequest {
                version: 1,
                members: members.clone(),
            },
        )
        .unwrap();
        let mut names = members.iter().cloned().collect::<BTreeSet<_>>();
        names.extend(outputs.iter().map(|o| o.path.clone()));
        let mut sources = BTreeMap::new();
        let mut saved_sources = BTreeMap::new();
        let mut changes = serde_json::Map::new();
        for path in names {
            let saved = super::super::resource_inputs::saved_target(root, &path)
                .unwrap()
                .1;
            let source = match &saved {
                SavedSource::Missing => SourceState::Missing { creatable: true },
                SavedSource::File { text, .. } => SourceState::File {
                    identity: path.clone(),
                    text: text.clone(),
                    writable: true,
                    removable: true,
                    version: None,
                },
            };
            sources.insert(path.clone(), source);
            saved_sources.insert(path.clone(), saved);
            changes.insert(
                crate::launch_profiles::inventory::workspace_source_uri(&path).unwrap(),
                json!([]),
            );
        }
        let capture = super::super::resource_inputs::Request {
            unaddressed_sources:BTreeMap::new(),
            context: "fixture".into(),
            edit: json!({"changes":changes}),
            drafts: BTreeMap::new(),
            project_entry: None,
            ownership_sources: BTreeMap::new(),
            saved_guards: BTreeMap::new(),
        };
        let inputs = Inputs {
            version: 1,
            context: "fixture".into(),
            workspace: "a".repeat(32),
            ownership: super::super::resource_inputs::Ownership {
                mode: "fixture".into(),
                entry: None,
                protected_entries: Vec::new(),
                saved_sources: BTreeMap::new(),
                draft_sources: BTreeMap::new(),
            },
            sources,
            saved_sources,
            refusals: BTreeMap::new(),
            saved_guards:BTreeMap::new(),
            source_files_modified: false,
        };
        let request = PrepareRequest {
            version: 1,
            reading: read,
            members,
            outputs,
            inputs,
            capture,
        };
        let plan = resource_plan(&request, &request.reading).unwrap();
        let proposal = super::super::resource_candidates::Request {
            inputs: request.inputs.clone(),
            capture: request.capture.clone(),
            plan,
        };
        let mut manifest =
            super::super::resource_journal::Manifest::from_review(&proposal.inputs, &proposal.plan)
                .unwrap();
        apply_modes(&mut manifest, &request.outputs);
        let mut full_after = proposal.inputs.saved_sources.clone();
        full_after.extend(manifest.after.clone());
        Prepared {
            request,
            proposal,
            manifest,
            facts: json!({}),
            full_after,
            fingerprint: "f".repeat(64),
            needs_save: true,
            stage_unavailable: None,
            published_guards:Mutex::new(None),
        }
    }
    fn text(path: &str, from: Option<&str>, body: &str) -> Destination {
        Destination {
            path: path.into(),
            identity_from: from.map(str::to_string),
            content: Content::Text { text: body.into() },
            mode: "100644".into(),
        }
    }
    fn install(root: &Path, p: &Prepared) {
        for (path, after) in &p.full_after {
            let before = super::super::resource_inputs::saved_target(root, path)
                .unwrap()
                .1;
            super::super::resource_publish::replace(root, path, &before, after).unwrap();
        }
    }
    #[test]
    fn real_composition_stages_exact_saved_bytes_and_preserves_unrelated_index() {
        let path = "literal[1] λ%?#.py";
        let t = fixture("text", path);
        let r = t.path();
        let p = prepared(r, vec![text(path, Some(path), "composed\n")]);
        assert!(legacy_unmerged(r, &[path.into()], false).unwrap());
        assert!(!legacy_unmerged(r, &["clean.py".into()], false).unwrap());
        let original = read_index(&repository(r).unwrap()).unwrap();
        install(r, &p);
        assert_eq!(original, read_index(&repository(r).unwrap()).unwrap());
        let result = atomic_stage(r, &p, || Ok(())).unwrap();
        assert_eq!(result["complete"], true);
        assert!(observe(&repository(r).unwrap()).unwrap().entries.is_empty());
        assert_eq!(
            git(r, &["show", &format!(":{path}")], None, None).unwrap(),
            b"composed\n"
        );
        assert_eq!(std::fs::read(r.join("clean.py")).unwrap(), b"clean\n");
    }
    #[test]
    fn delete_modify_and_explicit_rename_groups_never_stage_partial_members() {
        let t = fixture("delete", "case.py");
        let r = t.path();
        let p = prepared(r, Vec::new());
        install(r, &p);
        atomic_stage(r, &p, || Ok(())).unwrap();
        assert!(!r.join("case.py").exists());
        assert!(observe(&repository(r).unwrap()).unwrap().entries.is_empty());
        let t = fixture("rename", "old.py");
        let r = t.path();
        let p = prepared(
            r,
            vec![text("resolved.py", Some("left.py"), "resolved rename\n")],
        );
        assert_eq!(p.request.members.len(), 3);
        install(r, &p);
        atomic_stage(r, &p, || Ok(())).unwrap();
        assert!(
            !r.join("old.py").exists()
                && !r.join("left.py").exists()
                && !r.join("right.py").exists()
        );
        assert_eq!(
            std::fs::read(r.join("resolved.py")).unwrap(),
            b"resolved rename\n"
        );
        assert!(observe(&repository(r).unwrap()).unwrap().entries.is_empty());
    }
    #[test]
    fn index_source_and_admission_failures_leave_actual_index_untouched() {
        let t = fixture("text", "case.py");
        let r = t.path();
        let p = prepared(r, vec![text("case.py", Some("case.py"), "resolved\n")]);
        install(r, &p);
        let index = read_index(&repository(r).unwrap()).unwrap();
        assert!(atomic_stage(r, &p, || Err("injected preparation refusal".into())).is_err());
        assert_eq!(index, read_index(&repository(r).unwrap()).unwrap());
        std::fs::write(r.join("case.py"), "foreign edit\n").unwrap();
        assert!(atomic_stage(r, &p, || Ok(())).is_err());
        assert_eq!(index, read_index(&repository(r).unwrap()).unwrap());
        std::fs::write(r.join("clean.py"), "changed clean\n").unwrap();
        ok(r, &["add", "--", "clean.py"]);
        let changed = read_index(&repository(r).unwrap()).unwrap();
        assert!(atomic_stage(r, &p, || Ok(())).is_err());
        assert_eq!(changed, read_index(&repository(r).unwrap()).unwrap());
        assert!(!r.join(".git/index.lock").exists());
    }
    #[test]
    fn readable_incompatible_names_and_unsupported_filters_do_not_bypass_publication() {
        let t = fixture("text", "line\nname.py");
        let read = reading(
            t.path(),
            &ReadRequest {
                version: 1,
                members: vec!["line\nname.py".into()],
            },
        )
        .unwrap();
        let row = read
            .paths
            .iter()
            .find(|p| p.path == "line\nname.py")
            .unwrap();
        assert!(row.unavailable.is_some());
        assert!(row.stages.iter().all(|s| s.text.is_some()));
        let t = fixture("text", "case.py");
        let p = prepared(
            t.path(),
            vec![text("case.py", Some("case.py"), "resolved\n")],
        );
        std::fs::write(t.path().join(".gitattributes"), "case.py filter=custom\n").unwrap();
        assert!(
            stage_policy(
                &repository(t.path()).unwrap(),
                &p.request.outputs,
                &p.full_after
            )
            .is_err()
        );
        assert!(super::super::resource_changes::validate_path("../escape.py").is_err());
    }
}
</ui:copy>
<ui:file path="crates/hickory-cli/src/serve/git_resolutions.rs"><ui:paste select="#git-resolution-index"/><ui:paste select="#git-resolution-plan"/><ui:paste select="#git-resolution-service"/><ui:paste select="#git-resolution-tests"/></ui:file>

A completed refused staging request can retire only after a separate stage-phase validation proves the complete original index and reviewed saved-after namespace, including current generated consequences, under an owned index lease. This proof performs no source or index publication and never resets a consumed action. A fresh review is required. An unknown request or an already replaced index cannot pass this proof.

## Focused evidence and remaining acceptance

The canonical CLI library passed four real-Git index/model witnesses. A subsequently built engine passed seven owned HTTP scenarios: composed text with a literal Unicode/%?# filename, already-saved direct staging, delete/modify, explicit rename group, stale working source, stale index and a foreign index lock. Those runs exercised actual preparation, resource publication, retained status and staging; they did not render a frontend. The later stage-phase recovery proof and eighth stage-lock fixture are authored follow-ups pending the next generated build and execution. Controls/colon/backslash names remain readable but unavailable to the shared source publisher; custom clean filters/encodings and byte-changing CRLF staging remain explicit provider refusals. No cross-file crash atomicity or unexecuted device/accessibility acceptance is claimed.

## Drive the retained service through real Git and HTTP

The isolated runner constructs actual content, delete/modify and rename/rename conflicts, starts only owned engines, and exercises prepare, guarded publication and separate atomic staging over HTTP. It also verifies an already-saved group can stage directly, a changed working source or index refuses, and a foreign index lock remains intact. Completed refused requests expose typed retained status; a consumed review cannot become a second save. Optional Android mode drives the native controller against a literal filename, retaining the real editor while draft Undo/Redo, confirmed Save and explicit Stage execute. The renderer and physical input remain separate acceptance work.

<ui:copy id="git-resolution-service-runner">
#!/usr/bin/env python3
"""Exercise reviewed conflict publication against isolated real Git repositories and an owned engine."""
import argparse, importlib.util, json, os, subprocess, tempfile, urllib.error, urllib.parse, urllib.request, uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
PREFIX='/api/git/conflict-resolution/'
def git(root,*args,ok=True):
    p=subprocess.run(['git','-C',str(root),'--literal-pathspecs',*args],stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    if ok and p.returncode: raise RuntimeError(p.stderr.decode())
    return p.stdout

def make(root,kind):
    root.mkdir();git(root,'init','-q','-b','main');git(root,'config','user.name','Conflict fixture');git(root,'config','user.email','fixture@example.invalid')
    path='literal[1] λ%?#.py' if kind in ('text','noop','stale','index','lock','native') else 'case.py'
    (root/path).write_text('base\n');(root/'clean.py').write_text('clean\n');git(root,'add','--',path,'clean.py');git(root,'commit','-qm','base');git(root,'checkout','-qb','incoming')
    if kind=='rename':git(root,'mv','--',path,'right.py')
    else:(root/path).write_text('incoming\n')
    git(root,'add','-A');git(root,'commit','-qm','incoming');git(root,'checkout','-q','main')
    if kind=='rename':git(root,'mv','--',path,'left.py')
    elif kind=='delete':(root/path).unlink()
    else:(root/path).write_text('local\n')
    git(root,'add','-A');git(root,'commit','-qm','local');git(root,'merge','--no-edit','incoming',ok=False)
    assert git(root,'ls-files','-u','-z')
    return path

def request(origin,route,body=None):
    req=urllib.request.Request(origin+route,data=None if body is None else json.dumps(body).encode(),headers={'Content-Type':'application/json'})
    with urllib.request.urlopen(req,timeout=30) as response:return json.load(response)
def refused(action):
    try:action()
    except urllib.error.HTTPError as error:
        assert error.code in (400,409,422,503),error.read();return
    raise AssertionError('Unsafe action was admitted')
def prepare(origin,members,outputs):
    reading=request(origin,PREFIX+'read',{'version':1,'members':members})
    paths=sorted(set(members+[v['path'] for v in outputs]))
    capture={'context':uuid.uuid4().hex,'edit':{'changes':{'hick:///'+urllib.parse.quote(path,safe='/'):[] for path in paths}},'drafts':{},'project_entry':None,'ownership_sources':{}}
    inputs=request(origin,'/api/changes/resource-inputs',capture)
    return request(origin,PREFIX+'prepare',{'version':1,'reading':reading,'members':members,'outputs':outputs,'inputs':inputs,'capture':capture})
def output(path,identity,text):return {'path':path,'identity_from':identity,'mode':'100644','content':{'kind':'text','text':text}}
def act(origin,review,kind,identity=None):return request(origin,PREFIX+kind,{'version':1,'token':review['token'],'fingerprint':review['fingerprint'],'request_id':identity or uuid.uuid4().hex})
def suite(origin,root,kind,path):
    read=request(origin,PREFIX+'read',{'version':1,'members':[]});members=[row['path'] for row in read['paths']]
    if kind=='rename':outputs=[output('resolved.py','left.py','resolved rename\n')]
    elif kind=='delete':outputs=[]
    else:outputs=[output(path,path,(root/path).read_text() if kind=='noop' else 'composed = 41\n')]
    review=prepare(origin,members,outputs);assert review['resolution_protocol']==1
    before=(root/'.git/index').read_bytes();working={v:(root/v).read_bytes() if (root/v).exists() else None for v in members}
    if kind in ('stale','index','lock'):
        if kind=='stale':(root/path).write_text('foreign working edit\n')
        if kind=='index':(root/'clean.py').write_text('changed index\n');git(root,'add','--','clean.py');before=(root/'.git/index').read_bytes()
        if kind=='lock':(root/'.git/index.lock').write_text('foreign lock')
        identity=uuid.uuid4().hex;refused(lambda:act(origin,review,'save',identity));assert (root/'.git/index').read_bytes()==before
        status=request(origin,PREFIX+'status?request_id='+identity);assert status['state']=='completed' and status['outcome'] is None and status['error']
        if kind=='lock':
            assert (root/'.git/index.lock').read_text()=='foreign lock';(root/'.git/index.lock').unlink();assert request(origin,PREFIX+'validate',{'version':1,'token':review['token'],'fingerprint':review['fingerprint']})['current']
            refused(lambda:act(origin,review,'save'))
        else:refused(lambda:request(origin,PREFIX+'validate',{'version':1,'token':review['token'],'fingerprint':review['fingerprint']}))
        if kind!='stale':assert all(((root/v).read_bytes() if (root/v).exists() else None)==body for v,body in working.items())
        return
    if kind=='noop':assert not review['needs_save']
    else:
        identity=uuid.uuid4().hex;saved=act(origin,review,'save',identity);assert saved['request_id']==identity and saved['outcome']['complete'],saved
        assert saved['saved_sources']==review['saved_sources'];assert (root/'.git/index').read_bytes()==before
        assert request(origin,PREFIX+'status?request_id='+identity)['outcome']==saved
    if kind=='stage-lock':
        (root/'.git/index.lock').write_text('foreign stage lock');identity=uuid.uuid4().hex
        refused(lambda:act(origin,review,'stage',identity));assert (root/'.git/index').read_bytes()==before
        status=request(origin,PREFIX+'status?request_id='+identity);assert status['state']=='completed' and status['outcome'] is None
        assert (root/'.git/index.lock').read_text()=='foreign stage lock';(root/'.git/index.lock').unlink()
        proof={'version':1,'token':review['token'],'fingerprint':review['fingerprint'],'stage':True}
        assert request(origin,PREFIX+'validate',proof)['stage']
        refused(lambda:act(origin,review,'stage'))
        review=prepare(origin,members,outputs);assert not review['needs_save']
    identity=uuid.uuid4().hex;staged=act(origin,review,'stage',identity);assert staged['complete'] and not staged['index_uncertain'];assert staged['token']==review['token'] and staged['request_id']==identity and staged['review_fingerprint']==review['fingerprint']
    assert not git(root,'ls-files','-u','-z');assert git(root,'show',':clean.py')==b'clean\n'
    for value in outputs:assert git(root,'show',':'+value['path'])==value['content']['text'].encode()
    if kind=='delete':assert not (root/path).exists()
    if kind=='rename':assert not (root/'left.py').exists() and not (root/'right.py').exists() and not (root/'case.py').exists()

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--engine',type=Path,default=ROOT/'target/source-edit-focus/debug/examples/engine_client');parser.add_argument('--android',action='store_true');args=parser.parse_args()
    spec=importlib.util.spec_from_file_location('simple_fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
    with tempfile.TemporaryDirectory(prefix='hickory-git-resolutions-') as folder:
        temporary=Path(folder)
        for kind in ['text','noop','delete','rename','stale','index','lock','stage-lock']+(['native'] if args.android else []):
            root=temporary/kind;path=make(root,kind);process=fixture.start([str(args.engine.resolve()),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(temporary/(kind+'-state'))})
            try:
                origin=fixture.origin(process)
                if kind=='native':
                    env={**os.environ,'HICKORY_NATIVE_GIT_ORIGIN':origin,'HICKORY_NATIVE_GIT_WORKSPACE':str(root),'HICKORY_NATIVE_GIT_INPUT':path}
                    subprocess.run([str(ROOT/'target/simple-android-gradle'),'-Pandroid.builder.sdkDownload=false',':app:testDebugUnitTest','--tests','com.hickorydocs.simple.GitConflictResolutionTest.liveNativeCompositionSaveStageAndEditorHistory','--rerun-tasks'],cwd=ROOT,env=env,check=True)
                else:suite(origin,root,kind,path)
                print('PASS actual Git conflict '+kind,flush=True)
            finally:fixture.stop(process)
if __name__=='__main__':main()
</ui:copy>
<ui:file path="scripts/test-git-conflict-resolutions.py"><ui:paste select="#git-resolution-service-runner"/></ui:file>
</ui:doc>
