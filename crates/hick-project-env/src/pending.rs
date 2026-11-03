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
/// Explicitly adopt an already consistent source/output checkout, as after a pull.
/// This writes journal metadata only; ordinary generation still requires acceptance.
pub fn sync(root: &Path, entry: &str) -> Result<()> {
    let root = io(root.canonicalize())?;
    let current_sources = sources(&root, entry)?;
    let generated = evaluate_project(entry, &current_sources, &ProjectTree::default())?.new;
    let desired = Snapshot::new(current_sources.clone(), &generated);
    let store = Store::open(&root, entry)?;
    let pending = store.captures()?;
    // A mounted edit may never have reached disk. Do not archive it just because
    // the unrelated disk happens to agree with the source.
    if pending.iter().any(|c| c.virtual_output) {
        return Err(
            "mounted output edits remain pending; incorporate them explicitly before syncing"
                .into(),
        );
    }
    let before: Snapshot = if store.folder.join("baseline.json").exists() {
        store.read("baseline.json")?
    } else {
        desired.clone()
    };
    // Include newly constructed paths AND formerly constructed paths: a pulled
    // deletion must not hide a leftover local file. Read bytes, kinds and modes.
    let mut extra: Vec<_> = desired.entries.keys().cloned().collect();
    // Explicit captures may contain new files outside the old baseline and the
    // current construction. Those are local work too, not disposable history.
    extra.extend(
        pending
            .iter()
            .flat_map(|capture| capture.paths.iter().cloned()),
    );
    let after = before.disk(&root, &extra)?;
    let differences = generated.compare(&after.tree()?);
    if let Some(difference) = differences.first() {
        return Err(format!(
            "source/output mismatch at {}; sync refused without changing outputs or pending observations",
            difference.path
        ));
    }
    if sources(&root, entry)? != current_sources {
        return Err("source changed during synchronization".into());
    }
    if let Some(capture) = store.capture(entry, before, after)? {
        accept_from_store(&store, entry, &capture.id, &current_sources, &generated)?;
    } else {
        store.write("baseline.json", &desired)?;
    }
    Ok(())
}
/// Accept an authored arrangement, including amendments or a split among concepts.
/// Exact whole-project reconstruction and current disk bytes must both match the
/// chosen capture. All earlier captures remain archived, never silently discarded.
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
    accept_from_store(&store, entry, id, current_sources, generated)
}
fn accept_from_store(
    store: &Store,
    entry: &str,
    id: &str,
    current_sources: &BTreeMap<String, String>,
    generated: &ProjectTree,
) -> Result<()> {
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
#[cfg(test)]
#[path = "pending_tests.rs"]
mod tests;
