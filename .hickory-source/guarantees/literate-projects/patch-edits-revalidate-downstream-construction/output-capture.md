<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Capture output edits before deciding which concept owns them

For: authors using ordinary editors, formatters or agents on generated files.

A tool can tell us what it changed, but not which concept should explain the
change. Keep its exact predecessor and result in a durable pending capture.
Source provenance remains available to an explicit amendment; it does not grant
automatic permission to rewrite that source.

The journal is local under `.hick-journal/output-edits/`. Keep it with the working
tree until incorporation; deleting that directory discards local capture history.
Captures retain binary bytes, deletions and portable metadata even when a text
patch cannot represent them. New untracked products must be named explicitly.

Materialization takes a project lock and checks disk against the last successfully
published baseline before writing. Pending captures block publication, including
when source already reproduces disk. Incorporation checks the complete result,
not just the changed lines, and archives earlier observations only after an
explicit acceptance. Source changes are not automatically rebased.

The native guard is not a filesystem transaction against unrelated external
processes. A tool writing during publication can still race the final check;
edit at synchronization boundaries. A failed two-file concept proposal may leave
a detached document; the capture remains pending and output bytes remain intact.

## Retain observations without assigning conceptual ownership

A capture is an observation, not an executable project member. Store the source closure and portable output bytes with it so a later process can distinguish the original predecessor from a newly generated state.

<storycode:copy id="output-capture-types">
//! Durable capture of external output edits; conceptual ownership is explicit.
use fs2::FileExt;
use hick_lang::{EntryKind, ProjectEntry, ProjectTree, evaluate_project, load_project_sources};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Serialize, Deserialize)]
struct Entry {
    #[serde(with = "stored_bytes")]
    bytes: Vec<u8>,
    symlink: bool,
    executable: bool,
}
#[derive(Clone, Serialize, Deserialize)]
struct Snapshot {
    sources: BTreeMap<String, String>,
    entries: BTreeMap<String, Entry>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Capture {
    pub id: String,
    pub entry: String,
    pub paths: Vec<String>,
    /// None for changes that unified text patches cannot represent.
    pub patch: Option<String>,
    pub patch_error: Option<String>,
    /// A mounted view can include products never materialized on disk.
    #[serde(default)]
    virtual_output: bool,
    before: Snapshot,
    after: Snapshot,
}
struct Store {
    root: PathBuf,
    folder: PathBuf,
    _lock: fs::File,
}
fn io<T>(value: std::io::Result<T>) -> Result<T> {
    value.map_err(|e| e.to_string())
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

// Preserve arbitrary bytes while keeping the common UTF-8 case readable and
// compact. A large dogfood baseline should not expand text into integer arrays.
mod stored_bytes {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub fn serialize<S: Serializer>(
        bytes: &[u8],
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        match std::str::from_utf8(bytes) {
            Ok(text) => ("utf8", text).serialize(serializer),
            Err(_) => (
                "hex",
                bytes.iter().map(|b| format!("{b:02x}")).collect::<String>(),
            )
                .serialize(serializer),
        }
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Vec<u8>, D::Error> {
        let (kind, text) = <(String, String)>::deserialize(deserializer)?;
        match kind.as_str() {
            "utf8" => Ok(text.into_bytes()),
            "hex" if text.is_ascii() && text.len() % 2 == 0 => (0..text.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&text[i..i + 2], 16).map_err(serde::de::Error::custom))
                .collect(),
            _ => Err(serde::de::Error::custom("invalid stored output encoding")),
        }
    }
}
</storycode:copy>

## Read exact baselines through the native filesystem boundary

Read products as data and reject symlink ancestors before opening source or journal paths. Snapshot manifests share immutable, content-addressed source and output blobs, so repeated edits do not duplicate the whole dogfood repository. Blob reads verify their hashes. Journal updates use a sibling temporary file; the project lock serializes Hickory publication and capture operations. Text patches use small context hunks when supported; the exact snapshots remain authoritative.

### Confine native reads

A file symlink is data; a symlink ancestor can redirect a read outside the project. Check ancestors before reading bytes or source.

<storycode:copy id="output-confined-snapshots">
/// Refuse symlink ancestors, including in the journal. A link output may be
/// inspected as data, but source and journal files must be regular files.
fn confined(root: &Path, path: &str) -> Result<PathBuf> {
    hick_lang::validate_project_path(path)?;
    let mut full = root.to_path_buf();
    for part in path.split('/') {
        full.push(part);
        if full != root.join(path)
            && fs::symlink_metadata(&full).is_ok_and(|m| !m.is_dir() || m.file_type().is_symlink())
        {
            return Err(format!(
                "non-directory or symlink ancestor: {}",
                full.display()
            ));
        }
    }
    Ok(full)
}
fn regular(root: &Path, path: &str) -> Result<PathBuf> {
    let full = confined(root, path)?;
    if fs::symlink_metadata(&full).is_ok_and(|m| !m.is_file() || m.file_type().is_symlink()) {
        return Err(format!("not a regular file: {}", full.display()));
    }
    Ok(full)
}
fn read_entry(root: &Path, path: &str) -> Result<Option<Entry>> {
    let full = confined(root, path)?;
    let metadata = match fs::symlink_metadata(&full) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    let symlink = metadata.file_type().is_symlink();
    if !metadata.is_file() && !symlink {
        return Err(format!("output is not a file: {path}"));
    }
    let bytes = if symlink {
        io(fs::read_link(&full))?
            .to_str()
            .ok_or("non-UTF-8 link")?
            .as_bytes()
            .to_vec()
    } else {
        io(fs::read(&full))?
    };
    #[cfg(unix)]
    let executable = {
        use std::os::unix::fs::PermissionsExt;
        !symlink && metadata.permissions().mode() & 0o111 != 0
    };
    #[cfg(not(unix))]
    let executable = false;
    Ok(Some(Entry {
        bytes,
        symlink,
        executable,
    }))
}
</storycode:copy>

### Represent complete portable output

Snapshot comparison must include kinds and executable bits, not just visible text. Explicit extra paths let an author include a newly created product without sweeping unrelated working files into the capture.

<storycode:copy id="output-snapshot-values">
impl Snapshot {
    fn new(sources: BTreeMap<String, String>, tree: &ProjectTree) -> Self {
        Self {
            sources,
            entries: tree
                .entries()
                .iter()
                .map(|(p, e)| {
                    (
                        p.clone(),
                        Entry {
                            bytes: e.bytes.to_vec(),
                            symlink: e.kind == EntryKind::Symlink,
                            executable: e.executable,
                        },
                    )
                })
                .collect(),
        }
    }
    fn tree(&self) -> Result<ProjectTree> {
        let mut tree = ProjectTree::default();
        for (path, entry) in &self.entries {
            let mut value = ProjectEntry::file(entry.bytes.clone());
            value.kind = if entry.symlink {
                EntryKind::Symlink
            } else {
                EntryKind::File
            };
            value.executable = entry.executable;
            tree.insert(path, value)?;
        }
        Ok(tree)
    }
    fn disk(&self, root: &Path, extra: &[String]) -> Result<Self> {
        let mut result = self.clone();
        for path in self.entries.keys().chain(extra.iter()) {
            if self.sources.contains_key(path)
                || (path.starts_with(".hick-journal/") || path.starts_with(".hick-cache/"))
            {
                return Err(format!(
                    "capture path is source or journal metadata: {path}"
                ));
            }
            if let Some(entry) = read_entry(root, path)? {
                result.entries.insert(path.clone(), entry);
            } else {
                result.entries.remove(path);
            }
        }
        Ok(result)
    }
}
</storycode:copy>

### Serialize journal publication

Take one project journal lock before changing its manifests. A sibling temporary file keeps a reader from seeing partial JSON.

<storycode:copy id="output-journal-files">
impl Store {
    fn open(root: &Path, entry: &str) -> Result<Self> {
        hick_lang::validate_project_path(entry)?;
        let root = io(root.canonicalize())?;
        let journal = confined(&root, ".hick-journal/output-edits/lock")?;
        io(fs::create_dir_all(journal.parent().unwrap()))?;
        let journal = regular(&root, ".hick-journal/output-edits/lock")?;
        let lock = io(fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(journal))?;
        lock.lock_exclusive().map_err(|e| e.to_string())?;
        let relative = format!(".hick-journal/output-edits/{}", digest(entry.as_bytes()));
        let folder = confined(&root, &format!("{relative}/baseline.json"))?
            .parent()
            .unwrap()
            .to_path_buf();
        io(fs::create_dir_all(&folder))?;
        Ok(Self {
            root,
            folder,
            _lock: lock,
        })
    }
    fn read_value(&self, name: &str) -> Result<serde_json::Value> {
        let path = self.folder.join(name);
        let rel = path
            .strip_prefix(&self.root)
            .unwrap()
            .to_str()
            .ok_or("non-UTF-8 journal path")?;
        serde_json::from_slice(&io(fs::read(regular(&self.root, rel)?))?).map_err(|e| e.to_string())
    }
    fn write_value(&self, name: &str, value: &serde_json::Value) -> Result<()> {
        let path = self.folder.join(name);
        let rel = path
            .strip_prefix(&self.root)
            .unwrap()
            .to_str()
            .ok_or("non-UTF-8 journal path")?;
        let path = regular(&self.root, rel)?;
        io(fs::create_dir_all(path.parent().unwrap()))?;
        let mut temp = io(tempfile::NamedTempFile::new_in(path.parent().unwrap()))?;
        serde_json::to_writer(&mut temp, value).map_err(|e| e.to_string())?;
        io(temp.as_file().sync_all())?;
        temp.persist(path).map_err(|e| e.to_string())?;
        Ok(())
    }
</storycode:copy>

### Share immutable baseline content

Store each source and output value once, referenced by a manifest. Verify a blob when reading it, so corruption is an error rather than a different predecessor.

<storycode:copy id="output-journal-blobs">
    fn put_blob(&self, value: &serde_json::Value) -> Result<String> {
        let id = digest(&serde_json::to_vec(value).map_err(|e| e.to_string())?);
        let name = format!("blobs/{id}.json");
        // Read through the same confined path checks even for reusable blobs.
        let rel = self
            .folder
            .join(&name)
            .strip_prefix(&self.root)
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        let full = regular(&self.root, &rel)?;
        if !full.exists() {
            self.write_value(&name, value)?;
        }
        Ok(id)
    }
    fn blob(&self, id: &str) -> Result<serde_json::Value> {
        if id.len() != 64 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("invalid journal blob identity".into());
        }
        let value = self.read_value(&format!("blobs/{id}.json"))?;
        if digest(&serde_json::to_vec(&value).map_err(|e| e.to_string())?) != id {
            return Err("journal blob content changed".into());
        }
        Ok(value)
    }
    fn save_snapshot(&self, snapshot: &serde_json::Value) -> Result<serde_json::Value> {
        let mut manifest = serde_json::Map::new();
        for section in ["sources", "entries"] {
            let values = snapshot[section]
                .as_object()
                .ok_or("invalid snapshot section")?;
            let mut refs = serde_json::Map::new();
            for (path, value) in values {
                refs.insert(path.clone(), self.put_blob(value)?.into());
            }
            manifest.insert(section.into(), refs.into());
        }
        Ok(serde_json::json!({"snapshot":self.put_blob(&manifest.into())?}))
    }
    fn load_snapshot(&self, reference: &serde_json::Value) -> Result<serde_json::Value> {
        let Some(id) = reference.get("snapshot").and_then(|v| v.as_str()) else {
            return Ok(reference.clone());
        };
        let manifest = self.blob(id)?;
        let mut snapshot = serde_json::Map::new();
        for section in ["sources", "entries"] {
            let mut values = serde_json::Map::new();
            for (path, id) in manifest[section]
                .as_object()
                .ok_or("invalid snapshot manifest")?
            {
                values.insert(
                    path.clone(),
                    self.blob(id.as_str().ok_or("invalid snapshot blob")?)?,
                );
            }
            snapshot.insert(section.into(), values.into());
        }
        Ok(snapshot.into())
    }
</storycode:copy>

### Resolve manifests at the typed boundary

Baselines and captures retain their typed snapshots in memory. Resolve content references on read and replace them with immutable references on write.

<storycode:copy id="output-journal-records">
    fn read<T: serde::de::DeserializeOwned>(&self, name: &str) -> Result<T> {
        let mut value = self.read_value(name)?;
        if value.get("snapshot").is_some() {
            value = self.load_snapshot(&value)?;
        } else if value.get("before").is_some() {
            value["before"] = self.load_snapshot(&value["before"])?;
            value["after"] = self.load_snapshot(&value["after"])?;
        }
        serde_json::from_value(value).map_err(|e| e.to_string())
    }
    fn write<T: Serialize>(&self, name: &str, value: &T) -> Result<()> {
        let mut value = serde_json::to_value(value).map_err(|e| e.to_string())?;
        if value.get("sources").is_some() && value.get("entries").is_some() {
            value = self.save_snapshot(&value)?;
        } else if value.get("before").is_some() {
            value["before"] = self.save_snapshot(&value["before"])?;
            value["after"] = self.save_snapshot(&value["after"])?;
        }
        self.write_value(name, &value)
    }
    fn captures(&self) -> Result<Vec<Capture>> {
        let pending = self.folder.join("pending");
        // Validate the directory itself before enumerating it.
        let rel = pending.strip_prefix(&self.root).unwrap().to_str().unwrap();
        regular(&self.root, &format!("{rel}/check"))?;
        if !pending.exists() {
            return Ok(vec![]);
        }
        let mut captures = Vec::new();
        for file in io(fs::read_dir(pending))? {
            let file = io(file)?;
            let name = file
                .file_name()
                .into_string()
                .map_err(|_| "invalid journal filename")?;
            if name.ends_with(".json") {
                captures.push(self.read(&format!("pending/{name}"))?);
            }
        }
        captures.sort_by(|a: &Capture, b| a.id.cmp(&b.id));
        Ok(captures)
    }
</storycode:copy>

### Capture a transition without assigning ownership

Compare the exact predecessor and result. Validate a context diff by replaying it; use the evaluator's full-file form for empty-file edge cases. Unsupported binary or metadata diffs retain snapshots and an explicit explanation. A reverted output is still a selectable observation when older work is pending.

<storycode:copy id="output-observation-diffs">
    fn capture(&self, entry: &str, before: Snapshot, after: Snapshot) -> Result<Option<Capture>> {
        self.capture_kind(entry, before, after, false)
    }
    fn capture_kind(
        &self,
        entry: &str,
        before: Snapshot,
        after: Snapshot,
        virtual_output: bool,
    ) -> Result<Option<Capture>> {
        let old = before.tree()?;
        let new = after.tree()?;
        let paths: Vec<_> = old.compare(&new).into_iter().map(|d| d.path).collect();
        if paths.is_empty() && self.captures()?.is_empty() {
            return Ok(None);
        }
        let transition = hick_lang::unified_transition(&old, &new);
        let (patch, patch_error) = match transition {
            Ok(_) => {
                let mut patch = String::new();
                for path in &paths {
                    let a = before.entries.get(path);
                    let b = after.entries.get(path);
                    let old = a
                        .map(|e| std::str::from_utf8(&e.bytes))
                        .transpose()
                        .map_err(|e| e.to_string())?
                        .unwrap_or("");
                    let new = b
                        .map(|e| std::str::from_utf8(&e.bytes))
                        .transpose()
                        .map_err(|e| e.to_string())?
                        .unwrap_or("");
                    patch.push_str(
                        &similar::TextDiff::from_lines(old, new)
                            .unified_diff()
                            .context_radius(3)
                            .header(
                                &a.map_or("/dev/null".into(), |_| format!("a/{path}")),
                                &b.map_or("/dev/null".into(), |_| format!("b/{path}")),
                            )
                            .to_string(),
                    );
                }
                // Empty-file additions/deletions need the evaluator's exact zero-count form.
                if hick_lang::apply_project_patch(&old, &patch, "capture", 0)
                    .is_ok_and(|p| p.after.compare(&new).is_empty())
                {
                    (Some(patch), None)
                } else {
                    (Some(hick_lang::unified_transition(&old, &new)?), None)
                }
            }
            Err(e) => (None, Some(e)),
        };
        let identity =
            serde_json::to_vec(&(&before, &after, virtual_output)).map_err(|e| e.to_string())?;
        let capture = Capture {
            id: digest(&identity),
            entry: entry.into(),
            paths,
            patch,
            patch_error,
            before,
            after,
            virtual_output,
        };
        self.write(&format!("pending/{}.json", capture.id), &capture)?;
        self.write("current.json", &capture.id)?;
        Ok(Some(capture))
    }
}
impl Drop for Store {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self._lock);
    }
}
</storycode:copy>


## Stop regeneration before unresolved work can disappear

Capture against the last published bytes, even when Markdown has changed. Refuse a first materialization over unknown differences because no trustworthy predecessor exists. Acceptance allows a human or agent to amend or split concepts, but requires that the whole authored closure reproduce the selected observation and that disk still match.

### Guard generation before writing products

Capture disk against the last published snapshot before interpreting source changes. Unknown first-generation differences have no trustworthy baseline and must be reconciled explicitly.

<storycode:copy id="output-capture-publication">
/// Load exactly the declared Markdown closure. No generated working file is an input.
pub fn sources(root: &Path, entry: &str) -> Result<BTreeMap<String, String>> {
    load_project_sources(entry, &|p| io(fs::read_to_string(regular(root, p)?)))
}
pub fn status(root: &Path, entry: &str) -> Result<Vec<Capture>> {
    Store::open(root, entry)?.captures()
}
/// Capture current tracked products plus explicitly named new products. Capturing
/// never changes a document, output, or the baseline, even if source has moved.
pub fn capture(root: &Path, entry: &str, extra: &[String]) -> Result<Option<Capture>> {
    let store = Store::open(root, entry)?;
    let before: Snapshot = store.read("baseline.json").map_err(|e| {
        format!("No usable generation baseline; materialize the project first: {e}")
    })?;
    let after = before.disk(&store.root, extra)?;
    store.capture(entry, before, after)
}
/// First materialization refuses pre-existing unknown differences. Later runs
/// capture disk edits before considering source changes, and unresolved captures
/// block generation even if the source happens to have caught up.
pub fn materialize(
    root: &Path,
    entry: &str,
    sources: &BTreeMap<String, String>,
    tree: &ProjectTree,
) -> Result<Vec<PathBuf>> {
    let store = Store::open(root, entry)?;
    let desired = Snapshot::new(sources.clone(), tree);
    let baseline_path = store.folder.join("baseline.json");
    let baseline: Option<Snapshot> = if baseline_path.exists() {
        Some(store.read("baseline.json")?)
    } else {
        None
    };
    if let Some(before) = &baseline {
        store.capture(entry, before.clone(), before.disk(&store.root, &[])?)?;
    }
    let pending = store.captures()?;
    if !pending.is_empty() {
        return Err(format!(
            "{} pending output capture(s); generation stopped. Inspect with hick project edits {entry} --root {} (or behavior.py edits for dogfood), then incorporate explicitly.",
            pending.len(),
            store.root.display()
        ));
    }
    // Unknown outputs are never silently adopted as the predecessor of an edit.
    for (path, expected) in &desired.entries {
        if baseline
            .as_ref()
            .is_some_and(|b| b.entries.contains_key(path))
        {
            continue;
        }
        if let Some(actual) = read_entry(&store.root, path)? {
            let a = Snapshot {
                sources: BTreeMap::new(),
                entries: BTreeMap::from([(path.clone(), actual)]),
            }
            .tree()?;
            let b = Snapshot {
                sources: BTreeMap::new(),
                entries: BTreeMap::from([(path.clone(), expected.clone())]),
            }
            .tree()?;
            if !a.compare(&b).is_empty() {
                return Err(format!(
                    "no recorded baseline for differing output {path}; preserve it and reconcile before materializing"
                ));
            }
        }
    }
    let written = super::write_tree(&store.root, tree)?;
    if let Some(old) = baseline {
        for path in old
            .entries
            .keys()
            .filter(|p| !desired.entries.contains_key(*p))
        {
            io(fs::remove_file(confined(&store.root, path)?))?;
        }
    }
    store.write("baseline.json", &desired)?;
    Ok(written)
}
/// Accept an authored arrangement, including amendments or a split among concepts.
/// Exact whole-project reconstruction and current disk bytes must both match the
/// chosen capture. All earlier captures remain archived, never silently discarded.
</storycode:copy>

### Accept only an exact reconstructed result

An amended, split or newly patched source closure must reproduce the whole selected capture. Check disk again, then archive observations and update the baseline. Virtual captures check only their edited backing paths because untouched products may never have been materialized.

<storycode:copy id="output-capture-incorporation">
pub fn accept(root: &Path, entry: &str, id: &str) -> Result<()> {
    let root = io(root.canonicalize())?;
    let current_sources = sources(&root, entry)?;
    let evaluated = evaluate_project(entry, &current_sources, &ProjectTree::default())?;
    accept_evaluated(&root, entry, id, &current_sources, &evaluated.new)
}
/// Existing notebook pipelines can supply their fully evaluated output tree;
/// the same disk/source and capture guards apply at incorporation.
pub fn accept_evaluated(
    root: &Path,
    entry: &str,
    id: &str,
    current_sources: &BTreeMap<String, String>,
    generated: &ProjectTree,
) -> Result<()> {
    let store = Store::open(root, entry)?;
    let captures = store.captures()?;
    let capture = captures
        .iter()
        .find(|c| c.id == id)
        .ok_or("unknown pending capture")?;
    if sources(&store.root, entry)? != *current_sources {
        return Err("source changed before incorporation".into());
    }
    if !generated.compare(&capture.after.tree()?).is_empty() {
        return Err("authored source does not reproduce the captured whole-project result; capture remains pending".into());
    }
    let disk = capture.after.disk(&store.root, &[])?;
    let disk_mismatch = if capture.virtual_output {
        capture.paths.iter().any(
            |p| match (disk.entries.get(p), capture.after.entries.get(p)) {
                (Some(a), Some(b)) => {
                    a.bytes != b.bytes || a.symlink != b.symlink || a.executable != b.executable
                }
                (None, None) => false,
                _ => true,
            },
        )
    } else {
        !disk.tree()?.compare(&capture.after.tree()?).is_empty()
    };
    if disk_mismatch
        || capture.paths.iter().any(|p| {
            !capture.after.entries.contains_key(p)
                && fs::symlink_metadata(store.root.join(p)).is_ok()
        })
    {
        return Err("output changed since capture; capture again before accepting".into());
    }
    // Read source again after validation; never acknowledge a newer arrangement.
    if sources(&store.root, entry)? != *current_sources {
        return Err("source changed during incorporation".into());
    }
    if !capture.virtual_output || disk.tree()?.compare(generated).is_empty() {
        store.write(
            "baseline.json",
            &Snapshot::new(current_sources.clone(), generated),
        )?;
    }
    let archive = format!("accepted/{id}/check");
    let archive_path = regular(
        &store.root,
        store
            .folder
            .join(&archive)
            .strip_prefix(&store.root)
            .unwrap()
            .to_str()
            .unwrap(),
    )?;
    io(fs::create_dir_all(archive_path.parent().unwrap()))?;
    for capture in captures {
        io(fs::rename(
            store.folder.join(format!("pending/{}.json", capture.id)),
            archive_path
                .parent()
                .unwrap()
                .join(format!("{}.json", capture.id)),
        ))?;
    }
    store.write(
        "incorporated.json",
        &serde_json::json!({"entry":entry,"capture":id}),
    )?;
    Ok(())
}

/// A watcher uses the same baseline guard before its own per-file writer. No
/// captured path is overwritten merely because a document has changed too.
</storycode:copy>

### Keep watcher memory subordinate to durable state

Before its own writer runs, the watcher checks for pending observations. Only a completed generation with matching source and disk can advance its baseline.

<storycode:copy id="output-capture-watcher">
pub fn prepare_generation(
    root: &Path,
    entry: &str,
    source: &BTreeMap<String, String>,
    generated: &ProjectTree,
) -> Result<Vec<Capture>> {
    let store = Store::open(root, entry)?;
    if store.folder.join("baseline.json").exists() {
        let before: Snapshot = store.read("baseline.json")?;
        store.capture(entry, before.clone(), before.disk(&store.root, &[])?)?;
    } else {
        let pending = store.captures()?;
        if !pending.is_empty() {
            return Ok(pending);
        }
        let desired = Snapshot::new(source.clone(), generated);
        for (path, expected) in &desired.entries {
            if let Some(actual) = read_entry(&store.root, path)? {
                if actual.bytes != expected.bytes
                    || actual.symlink != expected.symlink
                    || actual.executable != expected.executable
                {
                    return Err(format!(
                        "no recorded baseline for differing output {path}; preserve it and reconcile before weaving"
                    ));
                }
            }
        }
    }
    store.captures()
}
/// Record a successful watcher write only after all products agree. If a tool
/// saved during weaving, capture that edit and keep the previous baseline.
pub fn checkpoint(
    root: &Path,
    entry: &str,
    source: &BTreeMap<String, String>,
    generated: &ProjectTree,
) -> Result<()> {
    let store = Store::open(root, entry)?;
    let snapshot = Snapshot::new(source.clone(), generated);
    if !store.captures()?.is_empty() {
        return Err("output edits remain pending; generation baseline was not advanced".into());
    }
    if !snapshot
        .disk(&store.root, &[])?
        .tree()?
        .compare(generated)
        .is_empty()
    {
        return Err("output changed during weaving; baseline was not advanced".into());
    }
    if sources(&store.root, entry)? != *source {
        return Err("source changed during weaving; baseline was not advanced".into());
    }
    store.write("baseline.json", &snapshot)
}

/// A virtual filesystem already has an exact read snapshot even if no backing
/// products exist. Preserve it without pretending all those files were on disk.
/// Repeated saves keep the first conceptual predecessor and expose a new result.
</storycode:copy>

### Capture mounted edits against their read snapshot

The mounted filesystem supplies a complete virtual read snapshot. Preserve the first conceptual predecessor across repeated saves, and expose the selected pending result without attaching its new bytes to old source provenance.

<storycode:copy id="output-capture-mounted">
pub fn capture_virtual(
    root: &Path,
    entry: &str,
    source: &BTreeMap<String, String>,
    before: &ProjectTree,
    after: &ProjectTree,
) -> Result<Capture> {
    let store = Store::open(root, entry)?;
    if sources(&store.root, entry)? != *source {
        return Err("source changed since the workspace read".into());
    }
    for (path, expected) in before.entries() {
        if let Some(actual) = read_entry(&store.root, path)? {
            if actual.bytes.as_slice() != expected.bytes.as_ref() {
                return Err(format!(
                    "output changed on disk since the workspace read: {path}"
                ));
            }
        }
    }
    let pending = store.captures()?;
    let predecessor = if let Some(first) = pending.first() {
        if first.before.sources != *source {
            return Err("pending capture has a different source baseline; incorporate it before another save".into());
        }
        first.before.clone()
    } else {
        Snapshot::new(source.clone(), before)
    };
    store
        .capture_kind(
            entry,
            predecessor,
            Snapshot::new(source.clone(), after),
            true,
        )?
        .ok_or("no virtual output change".into())
}
/// Return the selected observed result for mounted reads. Captures are immutable;
/// current.json selects the last observation, never a lexical ordering of hashes.
pub fn overlay(root: &Path, entry: &str) -> Result<BTreeMap<String, Option<Vec<u8>>>> {
    let store = Store::open(root, entry)?;
    if store.captures()?.is_empty() {
        return Ok(BTreeMap::new());
    }
    let id: String = store.read("current.json")?;
    if id.len() != 64 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid current capture identity".into());
    }
    let capture: Capture = store.read(&format!("pending/{id}.json"))?;
    Ok(capture
        .paths
        .iter()
        .map(|p| {
            (
                p.clone(),
                capture.after.entries.get(p).map(|e| e.bytes.clone()),
            )
        })
        .collect())
}
</storycode:copy>


## Author a new conceptual transformation only by explicit choice

A new concept initially depends on all constructions in the predecessor. That deliberately conservative ordering avoids inferring semantic independence from a textual diff. Validate the new closure before creating the concept and entry include. The separate acceptance step leaves a reviewable pending change.

<storycode:copy id="output-concept-proposal">
fn attribute(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
/// Author a new concept patch and include it in the entry. The capture stays
/// pending for review until `accept`. No source is rewritten by provenance.
/// Conservative predecessors include every construction in the captured state;
/// authors can narrow them later, subject to full reconstruction validation.
pub fn new_patch(
    root: &Path,
    entry: &str,
    id: &str,
    document: &str,
    patch_id: &str,
    explanation: &str,
) -> Result<()> {
    let store = Store::open(root, entry)?;
    let captures = store.captures()?;
    let capture = captures
        .iter()
        .find(|c| c.id == id)
        .ok_or("unknown pending capture")?;
    let diff = capture.patch.as_ref().ok_or_else(|| {
        capture
            .patch_error
            .clone()
            .unwrap_or("not a textual patch".into())
    })?;
    if diff.is_empty() {
        return Err(
            "capture has no output difference; accept the reverted observation explicitly".into(),
        );
    }
    if patch_id.is_empty()
        || patch_id.chars().any(char::is_whitespace)
        || explanation.trim().is_empty()
    {
        return Err("a stable patch id and an explanation are required".into());
    }
    let mut current = sources(&store.root, entry)?;
    if current != capture.before.sources {
        return Err("source changed since the captured baseline; reconcile it explicitly before proposing a new patch".into());
    }
    let evaluated = evaluate_project(entry, &current, &ProjectTree::default())?;
    if evaluated.local_states.contains_key(patch_id) {
        return Err("patch id already exists".into());
    }
    let path = regular(&store.root, document)?;
    if path.exists()
        || capture.after.entries.contains_key(document)
        || current.contains_key(document)
    {
        return Err("new concept document already exists or overlaps an output".into());
    }
    let parent = entry.rsplit_once('/').map_or("", |(dir, _)| dir);
    let relative = if parent.is_empty() {
        document
    } else {
        document
            .strip_prefix(&format!("{parent}/"))
            .ok_or("new concept must be below the entry directory")?
    };
    let mut prefix = "capture".to_owned();
    while diff.contains(&format!("</{prefix}:patch>"))
        || explanation.contains(&format!("<{prefix}:"))
        || explanation.contains(&format!("</{prefix}:"))
    {
        prefix.push('_');
    }
    let predecessors = evaluated
        .local_states
        .keys()
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    let concept = format!(
        "<{prefix}:doc xmlns:{prefix}=\"http://www.hickorydocs.com/1.0\">\n# {}\n\n{}\n\n<{prefix}:patch id=\"{}\" after=\"{}\">\n{diff}</{prefix}:patch>\n</{prefix}:doc>\n",
        patch_id,
        explanation,
        attribute(patch_id),
        attribute(&predecessors)
    );
    let source = &current[entry];
    let parsed = hick_lang::parse(source).map_err(|e| e.to_string())?;
    let include = format!(
        "\n<{}:include file=\"{}\" />\n",
        parsed.prefix,
        attribute(relative)
    );
    let close = format!("</{}:doc>", parsed.prefix);
    let mut updated = source.clone();
    let offset = updated.rfind(&close).unwrap_or(updated.len());
    updated.insert_str(offset, &include);
    current.insert(entry.into(), updated.clone());
    current.insert(document.into(), concept.clone());
    let candidate = evaluate_project(entry, &current, &ProjectTree::default())?;
    if !candidate.new.compare(&capture.after.tree()?).is_empty() {
        return Err("new concept did not reconstruct captured result".into());
    }
    if sources(&store.root, entry)? != capture.before.sources {
        return Err("source changed during proposal validation".into());
    }
    io(fs::create_dir_all(path.parent().unwrap()))?;
    // Create-new prevents replacing a concept written during validation.
    use std::io::Write;
    let mut file = io(fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path))?;
    io(file.write_all(concept.as_bytes()))?;
    // A failed second write leaves a detached concept plus the pending capture;
    // it cannot acknowledge or destroy the external edit.
    let entry_path = regular(&store.root, entry)?;
    let mut temp = io(tempfile::NamedTempFile::new_in(
        entry_path.parent().unwrap(),
    ))?;
    io(temp.write_all(updated.as_bytes()))?;
    temp.persist(entry_path).map_err(|e| e.to_string())?;
    Ok(())
}
</storycode:copy>

## Exercise external tools, incorporation, staleness and refusal

These focused witnesses use ordinary filesystem writes as the naive external tool. They check restart-readable captures, unchanged original ownership, refusal of stale choices, amendments and split ownership, binary and deleted products, and confinement.

### Prepare an ordinary-tool fixture

Start from a real materialized Python product and a recorded baseline. Tests change disk through ordinary filesystem operations.

<storycode:copy id="output-capture-tests">
#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (tempfile::TempDir, BTreeMap<String, String>, ProjectTree) {
        let root = tempfile::tempdir().unwrap();
        let source = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project/><h:file id=\"base\" path=\"app.py\">print('hello')\n</h:file></h:doc>";
        fs::write(root.path().join("project.md"), source).unwrap();
        let sources = sources(root.path(), "project.md").unwrap();
        let tree = evaluate_project("project.md", &sources, &ProjectTree::default())
            .unwrap()
            .new;
        materialize(root.path(), "project.md", &sources, &tree).unwrap();
        (root, sources, tree)
    }</storycode:copy>

### external edits are durable pending and block generation until explicit incorporation

The assertions below exercise this outcome against source, disk and the durable journal.

<storycode:copy id="output-test-external-edits-are-durable-pending-and-block-generation-until-explicit-incorporation">

    #[test]
    fn external_edits_are_durable_pending_and_block_generation_until_explicit_incorporation() {
        let (root, original, tree) = setup();
        fs::write(root.path().join("app.py"), "print('redacted')\n").unwrap();
        let error = materialize(root.path(), "project.md", &original, &tree).unwrap_err();
        assert!(error.contains("pending output"));
        assert_eq!(sources(root.path(), "project.md").unwrap(), original);
        assert_eq!(
            fs::read_to_string(root.path().join("app.py")).unwrap(),
            "print('redacted')\n"
        );
        let pending = status(root.path(), "project.md").unwrap();
        assert_eq!(pending.len(), 1);
        let id = &pending[0].id;
        assert!(
            pending[0]
                .patch
                .as_ref()
                .unwrap()
                .contains("+print('redacted')")
        );
        assert_eq!(
            capture(root.path(), "project.md", &[]).unwrap().unwrap().id,
            *id
        );
        assert!(
            accept(root.path(), "project.md", id)
                .unwrap_err()
                .contains("does not reproduce")
        );
        new_patch(
            root.path(),
            "project.md",
            id,
            "privacy.md",
            "redact",
            "Avoid disclosing the original greeting.",
        )
        .unwrap();
        assert!(status(root.path(), "project.md").unwrap().len() == 1);
        assert!(
            fs::read_to_string(root.path().join("project.md"))
                .unwrap()
                .contains("print('hello')")
        );
        let concept = fs::read_to_string(root.path().join("privacy.md")).unwrap();
        assert!(concept.contains("after=\"base\""));
        accept(root.path(), "project.md", id).unwrap();
        assert!(status(root.path(), "project.md").unwrap().is_empty());
        let authored = sources(root.path(), "project.md").unwrap();
        let output = evaluate_project("project.md", &authored, &ProjectTree::default())
            .unwrap()
            .new;
        materialize(root.path(), "project.md", &authored, &output).unwrap();
    }</storycode:copy>

### moved source and later output edits cannot be implicitly acknowledged

The assertions below exercise this outcome against source, disk and the durable journal.

<storycode:copy id="output-test-moved-source-and-later-output-edits-cannot-be-implicitly-acknowledged">

    #[test]
    fn moved_source_and_later_output_edits_cannot_be_implicitly_acknowledged() {
        let (root, original, tree) = setup();
        fs::write(root.path().join("app.py"), "print('first')\n").unwrap();
        let first = capture(root.path(), "project.md", &[]).unwrap().unwrap();
        fs::write(
            root.path().join("project.md"),
            original["project.md"].replace("hello", "first"),
        )
        .unwrap();
        assert!(
            new_patch(
                root.path(),
                "project.md",
                &first.id,
                "concept.md",
                "change",
                "Reason"
            )
            .unwrap_err()
            .contains("source changed")
        );
        let edited_sources = sources(root.path(), "project.md").unwrap();
        let edited_tree = evaluate_project("project.md", &edited_sources, &ProjectTree::default())
            .unwrap()
            .new;
        assert!(materialize(root.path(), "project.md", &edited_sources, &edited_tree).is_err());
        fs::write(root.path().join("app.py"), "print('second')\n").unwrap();
        assert!(
            accept(root.path(), "project.md", &first.id)
                .unwrap_err()
                .contains("output changed")
        );
        assert_eq!(status(root.path(), "project.md").unwrap().len(), 1);
        assert!(
            tree.get("app.py")
                .unwrap()
                .bytes
                .starts_with(b"print('hello')")
        );
    }</storycode:copy>

### an amended or split arrangement is accepted only for the complete capture

The assertions below exercise this outcome against source, disk and the durable journal.

<storycode:copy id="output-test-an-amended-or-split-arrangement-is-accepted-only-for-the-complete-capture">

    #[test]
    fn an_amended_or_split_arrangement_is_accepted_only_for_the_complete_capture() {
        let (root, original, _) = setup();
        fs::write(root.path().join("app.py"), "print('new')\n").unwrap();
        fs::write(root.path().join("helper.py"), "value = 1\n").unwrap();
        let edit = capture(root.path(), "project.md", &["helper.py".into()])
            .unwrap()
            .unwrap();
        let amended = original["project.md"].replace("hello", "new");
        fs::write(root.path().join("project.md"), &amended).unwrap();
        assert!(accept(root.path(), "project.md", &edit.id).is_err());
        let split = amended.replace("</h:doc>", "<h:include file=\"helper.md\"/></h:doc>");
        fs::write(root.path().join("project.md"), split).unwrap();
        fs::write(root.path().join("helper.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"helper.py\">value = 1\n</h:file></h:doc>").unwrap();
        accept(root.path(), "project.md", &edit.id).unwrap();
    }</storycode:copy>

### binary and deleted outputs are preserved without inventing text patches

The assertions below exercise this outcome against source, disk and the durable journal.

<storycode:copy id="output-test-binary-and-deleted-outputs-are-preserved-without-inventing-text-patches">

    #[test]
    fn binary_and_deleted_outputs_are_preserved_without_inventing_text_patches() {
        let (root, _, _) = setup();
        fs::write(root.path().join("app.py"), [0xff, 0x00]).unwrap();
        let edit = capture(root.path(), "project.md", &[]).unwrap().unwrap();
        assert!(edit.patch.is_none());
        assert!(edit.patch_error.unwrap().contains("binary"));
        assert_eq!(status(root.path(), "project.md").unwrap().len(), 1);
        fs::remove_file(root.path().join("app.py")).unwrap();
        let deletion = capture(root.path(), "project.md", &[]).unwrap().unwrap();
        assert!(deletion.patch.unwrap().contains("+++ /dev/null"));
        assert!(accept(root.path(), "project.md", &deletion.id).is_err());
        assert!(!root.path().join("app.py").exists());
    }</storycode:copy>

### virtual captures preserve unmaterialized products without claiming disk writes

The assertions below exercise this outcome against source, disk and the durable journal.

<storycode:copy id="output-test-virtual-captures-preserve-unmaterialized-products-without-claiming-disk-writes">

    #[test]
    fn virtual_captures_preserve_unmaterialized_products_without_claiming_disk_writes() {
        let (root, source, tree) = setup();
        let mut before = tree.clone();
        before
            .insert("virtual.txt", ProjectEntry::file(b"not on disk\n".to_vec()))
            .unwrap();
        let mut after = before.clone();
        after
            .insert("app.py", ProjectEntry::file(b"print('mounted')\n".to_vec()))
            .unwrap();
        let capture = capture_virtual(root.path(), "project.md", &source, &before, &after).unwrap();
        assert!(!root.path().join("virtual.txt").exists());
        fs::write(root.path().join("app.py"), "print('mounted')\n").unwrap();
        let overlay = overlay(root.path(), "project.md").unwrap();
        assert_eq!(
            overlay["app.py"].as_deref(),
            Some(b"print('mounted')\n".as_slice())
        );
        // A supplied weave must still match the whole virtual result.
        assert!(accept_evaluated(root.path(), "project.md", &capture.id, &source, &tree).is_err());
        accept_evaluated(root.path(), "project.md", &capture.id, &source, &after).unwrap();
        assert!(status(root.path(), "project.md").unwrap().is_empty());
        assert!(!root.path().join("virtual.txt").exists());
    }</storycode:copy>

### reverting disk still requires explicit acceptance and preserves history

The assertions below exercise this outcome against source, disk and the durable journal.

<storycode:copy id="output-test-reverting-disk-still-requires-explicit-acceptance-and-preserves-history">

    #[test]
    fn reverting_disk_still_requires_explicit_acceptance_and_preserves_history() {
        let (root, source, tree) = setup();
        fs::write(root.path().join("app.py"), "changed\n").unwrap();
        let changed = capture(root.path(), "project.md", &[]).unwrap().unwrap();
        fs::write(root.path().join("app.py"), "print('hello')\n").unwrap();
        let reverted = capture(root.path(), "project.md", &[]).unwrap().unwrap();
        assert_ne!(changed.id, reverted.id);
        assert!(reverted.paths.is_empty());
        assert!(materialize(root.path(), "project.md", &source, &tree).is_err());
        accept(root.path(), "project.md", &reverted.id).unwrap();
        assert!(status(root.path(), "project.md").unwrap().is_empty());
    }</storycode:copy>

### capture and publication refuse symlink ancestors and unknown preexisting outputs

The assertions below exercise this outcome against source, disk and the durable journal.

<storycode:copy id="output-test-capture-and-publication-refuse-symlink-ancestors-and-unknown-preexisting-outputs">

    #[cfg(unix)]
    #[test]
    fn capture_and_publication_refuse_symlink_ancestors_and_unknown_preexisting_outputs() {
        let (root, sources, tree) = setup();
        let outside = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        fs::write(other.path().join("app.py"), "unrelated work").unwrap();
        assert!(
            materialize(other.path(), "project.md", &sources, &tree)
                .unwrap_err()
                .contains("no recorded baseline")
        );
        assert_eq!(
            fs::read_to_string(other.path().join("app.py")).unwrap(),
            "unrelated work"
        );
        std::os::unix::fs::symlink(outside.path(), root.path().join("linked")).unwrap();
        assert!(capture(root.path(), "project.md", &["linked/secret".into()]).is_err());
        fs::rename(
            root.path().join(".hick-journal"),
            root.path().join("journal-backup"),
        )
        .unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join(".hick-journal")).unwrap();
        assert!(status(root.path(), "project.md").is_err());
        assert!(!outside.path().join("output-edits").exists());
    }
}
</storycode:copy>


## Exercise the public CLI across concept pages

The author starts with foundation and logging documents contributing to the same
Python file. An ordinary filesystem write must not amend either page. The CLI
then authors a privacy concept, retains the pending decision for review, and
accepts only after complete reconstruction. This also protects the existing pure
`verify` command while adding working-tree synchronization commands.

<storycode:copy id="output-test-cli-concept-incorporation">
// Capture external tools' edits without choosing an existing concept as owner.
#[test]
fn cli_captures_a_multi_document_edit_as_a_new_concept_before_acceptance() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("project.md"), "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"foundation.md logging.md\"/></h:doc>").unwrap();
    let foundation = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file id=\"base\" path=\"app.py\">print('hello')\n</h:file></h:doc>";
    let logging = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:patch id=\"logging\" after=\"base\">\n--- a/app.py\n+++ b/app.py\n@@ -1 +1 @@\n-print('hello')\n+print('logged')\n</h:patch></h:doc>";
    std::fs::write(root.join("foundation.md"), foundation).unwrap();
    std::fs::write(root.join("logging.md"), logging).unwrap();
    let run = |args: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_hick"))
            .arg("project")
            .args(args)
            .arg("--root")
            .arg(root)
            .output()
            .unwrap()
    };
    let success = |args: &[&str]| {
        let output = run(args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    };
    success(&["materialize", "project.md"]);
    assert_eq!(
        std::fs::read_to_string(root.join("app.py")).unwrap(),
        "print('logged')\n"
    );
    // This is exactly what a naive editor or formatter does: write a file.
    std::fs::write(root.join("app.py"), "print('private')\n").unwrap();
    let refused = run(&["materialize", "project.md"]);
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("pending output"));
    assert_eq!(
        std::fs::read_to_string(root.join("foundation.md")).unwrap(),
        foundation
    );
    assert_eq!(
        std::fs::read_to_string(root.join("logging.md")).unwrap(),
        logging
    );
    success(&["capture", "project.md"]);
    let listed = success(&["edits", "project.md"]);
    let id = listed.split_once(':').unwrap().0;
    let patch = success(&["edits", "project.md", "--id", id]);
    assert!(patch.contains("+print('private')"));
    success(&[
        "new-patch",
        "project.md",
        "--id",
        id,
        "--document",
        "privacy.md",
        "--patch-id",
        "privacy",
        "--explanation",
        "Keep sensitive values out of logs.",
    ]);
    assert_eq!(
        std::fs::read_to_string(root.join("logging.md")).unwrap(),
        logging
    );
    assert!(
        std::fs::read_to_string(root.join("privacy.md"))
            .unwrap()
            .contains("Keep sensitive values out of logs.")
    );
    assert!(success(&["edits", "project.md"]).contains(id));
    success(&["accept", "project.md", "--id", id]);
    assert!(success(&["edits", "project.md"]).contains("No pending captures"));
    success(&["verify", "project.md"]);
    success(&["materialize", "project.md"]);
    assert_eq!(
        std::fs::read_to_string(root.join("app.py")).unwrap(),
        "print('private')\n"
    );
}
</storycode:copy>

## Executed focused checks — 2026-10-27

The following commands passed against the regenerated implementation. These are
focused behavior and safety witnesses, not a broad regression run or human
acceptance of the complete application.

- `cargo test --offline --locked -p hick-project-env --lib pending::tests` — 7 passed.
- `cargo test --offline --locked -p hickory-cli --lib up::watcher_mode_tests` — 2 passed, including acceptance waking the watcher and retention across restart.
- `cargo test --offline --locked -p hickory-cli --lib up::state::resolution_tests` — 3 passed.
- `cargo test --offline --locked -p hickory-cli --test project_revision cli_captures_a_multi_document_edit_as_a_new_concept_before_acceptance -- --exact` — 1 passed.
- `cargo test --offline --locked -p hickory-cli --test workspace_fs` — 8 passed.
- `HICKORY_WATCH_MODE=poll cargo test --offline --locked -p hickory-cli --test up_loop an_edit_saved_in_a_woven_file_lands_in_the_document -- --exact` — 1 passed with an isolated `HICKORY_STATE_DIR`.

The last witness initially timed out with native watching. A direct reproduction
received ordinary writes but missed an early atomic replacement in this host.
Polling delivered that replacement. Native atomic-save event delivery remains
unverified here; this change does not claim to repair the host's native watcher.
The generation preflight checks actual bytes independently of event delivery.

Two temporary-directory smoke flows also passed: the dogfood Python wrapper's
`generate → baseline → external edit → capture → new-patch → accept → check`
path, and a real default `hick up` process retaining an atomic save and refreshing
its hold after CLI acceptance, using polling. No source owner was automatically
amended in either default capture flow.

Empty-tree reconstruction, `just literate-bootstrap`, source/output `check`, and
ownership `audit` passed. The existing explanation gaps remain recorded in the
narrative map; reconstruction does not establish explanatory completeness.

## Assemble the native capture mechanism

<storycode:file path="crates/hick-project-env/src/pending.rs"><storycode:paste select="#output-capture-types"/><storycode:paste select="#output-confined-snapshots"/><storycode:paste select="#output-snapshot-values"/><storycode:paste select="#output-journal-files"/><storycode:paste select="#output-journal-blobs"/><storycode:paste select="#output-journal-records"/><storycode:paste select="#output-observation-diffs"/><storycode:paste select="#output-capture-publication"/><storycode:paste select="#output-capture-incorporation"/><storycode:paste select="#output-capture-watcher"/><storycode:paste select="#output-capture-mounted"/><storycode:paste select="#output-concept-proposal"/><storycode:paste select="#output-capture-tests"/><storycode:paste select="#output-test-external-edits-are-durable-pending-and-block-generation-until-explicit-incorporation"/><storycode:paste select="#output-test-moved-source-and-later-output-edits-cannot-be-implicitly-acknowledged"/><storycode:paste select="#output-test-an-amended-or-split-arrangement-is-accepted-only-for-the-complete-capture"/><storycode:paste select="#output-test-binary-and-deleted-outputs-are-preserved-without-inventing-text-patches"/><storycode:paste select="#output-test-virtual-captures-preserve-unmaterialized-products-without-claiming-disk-writes"/><storycode:paste select="#output-test-reverting-disk-still-requires-explicit-acceptance-and-preserves-history"/><storycode:paste select="#output-test-capture-and-publication-refuse-symlink-ancestors-and-unknown-preexisting-outputs"/></storycode:file>

</storycode:doc>
