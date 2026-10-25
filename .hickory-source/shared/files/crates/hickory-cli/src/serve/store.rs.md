<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/store.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Persist local document edits in ordinary files](../../../../../../guarantees/collaboration/local-session-writes-your-files/contract.md)
- [The App Runs The Up-Loop: External Edits Are Seen, Never Clobbered](../../../../../../guarantees/collaboration/the-app-sees-external-edits/contract.md)

Captured SHA-256: `b9e8bf491ad9b9e518b49a6b8c2b1ba477e2c23b16eb21c6e08652458db041d9`.

<book:entry path="crates/hickory-cli/src/serve/store.rs">
//! The local [`DocStore`]: a room's durable state is the file itself.
//!
//! The hosted server keeps a room's text in a Postgres row and its encoded
//! CRDT state in a column beside it. Here the text *is* the `.md` file on
//! disk — which is the point of the whole mode: a collaborator's keystroke
//! lands in the host's working tree, where their editor, their `git diff`, and
//! `hick test` all see it. The CRDT state, which no human reads, goes in a
//! sidecar under `.hick-cache/` (already gitignored by `hick init`).
//!
//! See `docs/specs/freeform/local-collaboration.md`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context as _, Result};
use async_trait::async_trait;
use hickory_collab::{DocKey, DocStore};

/// Every `.md` document under the served root, keyed by a stable id.
///
/// Ids are derived from the path rather than minted, so a share link still
/// resolves after the host restarts `hick serve` — a link that dies when
/// the laptop sleeps is not a link.
pub struct DocIndex {
    root: PathBuf,
    absolute_ids: bool,
    /// id → path relative to the root.
    ///
    /// Behind a lock because a document can be created while the app is
    /// running: the scan happens once at startup, and a new file that nothing
    /// knows about is a file nothing can open.
    by_id: std::sync::RwLock<HashMap<String, String>>,
}

impl DocIndex {
    /// Scan `root` for documents. A single file may be served directly, in
    /// which case its parent directory is the root.
    pub fn scan(root: &Path) -> Result<Self> {
        // Empty folders are valid native workspaces. Use the same default
        // discovery policy as CLI expansion, without a second cache scan.
        // Explicit configuration continues through its own resolver unchanged.
        let paths = if root.is_dir() && !root.join("_hick.yml").is_file() {
            hick_literate::discover_default_documents(root)?
        } else {
            crate::expand_docs(root)?
        };
        let mut by_id = HashMap::new();
        for path in paths {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            by_id.insert(doc_id(&rel), rel);
        }
        Ok(Self {
            root: root.to_path_buf(),
            absolute_ids: false,
            by_id: std::sync::RwLock::new(by_id),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Shared engine views use the physical path as their room identity,
    /// so opening a parent and its child cannot mint two rooms for one file.
    pub fn shared(root: &Path) -> Result<Self> {
        Ok(Self::shared_from(&Self::scan(root)?))
    }

    /// Reuse a completed scan when adopting physical room identities.
    pub fn shared_from(scanned: &Self) -> Self {
        let index = Self::empty_shared(scanned.root());
        for (_, path) in scanned.entries() {
            index.add(&path);
        }
        index
    }

    /// Selected-file tracking in a window proxy needs no workspace scan;
    /// the engine owns discovery and populates its own shared index.
    pub fn empty_shared(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            absolute_ids: true,
            by_id: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Relative path for a document id.
    pub fn path_of(&self, id: &str) -> Option<String> {
        self.by_id.read().ok()?.get(id).cloned()
    }

    /// Remember a document created while the server was running, and return
    /// its id.
    pub fn add(&self, rel: &str) -> String {
        let id = self.id_for_path(rel);
        if let Ok(mut map) = self.by_id.write() {
            map.insert(id.clone(), rel.to_string());
        }
        id
    }

    /// Register a background-discovered document without treating runtime
    /// Markdown as project source. Explicit opens/writes continue to use add.
    pub fn discover(&self, rel: &str) -> Option<String> {
        let id = self.id_for_path(rel);
        if self.by_id.read().ok()?.contains_key(&id) {
            return Some(id); // Keep explicitly opened/configured documents live.
        }
        let path = self.root.join(rel);
        let eligible = if self.root.join("_hick.yml").is_file() {
            crate::expand_docs(&self.root)
                .ok()?
                .iter()
                .any(|configured| configured == &path)
        } else {
            hick_literate::is_default_document_path(&self.root, &path)
        };
        eligible.then(|| self.add(rel))
    }

    /// Absolute path for a document id.
    pub fn absolute(&self, id: &str) -> Option<PathBuf> {
        self.path_of(id).map(|rel| self.root.join(rel))
    }

    /// Id for a path relative to the root, whether or not it was scanned —
    /// provenance can name an upstream document the scan did not reach.
    pub fn id_for_path(&self, rel: &str) -> String {
        if self.absolute_ids {
            let path = self.root.join(rel);
            let path = path.canonicalize().unwrap_or(path);
            doc_id(&path.to_string_lossy())
        } else {
            doc_id(rel)
        }
    }

    /// Every document, as `(id, relative path)`, sorted by path.
    pub fn entries(&self) -> Vec<(String, String)> {
        let Ok(map) = self.by_id.read() else {
            return Vec::new();
        };
        let mut out: Vec<(String, String)> = map
            .iter()
            .map(|(id, rel)| (id.clone(), rel.clone()))
            .collect();
        out.sort_by(|a, b| a.1.cmp(&b.1));
        out
    }

    /// The single document, when exactly one was found. `hick serve
    /// doc.md` opens straight into it rather than a one-item list.
    pub fn sole(&self) -> Option<(String, String)> {
        match self.entries().as_slice() {
            [only] => Some(only.clone()),
            _ => None,
        }
    }
}

/// Stable, short, filesystem-safe id for a relative document path.
///
/// FNV-1a in hex: deterministic across machines and restarts, no state to
/// keep, and short enough to read in a URL.
pub fn doc_id(rel_path: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in rel_path.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    format!("{hash:016x}")
}

/// Files on disk, standing in for Postgres.
pub struct FileDocStore {
    index: Arc<DocIndex>,
    paths: std::sync::RwLock<HashMap<String, PathBuf>>,
    /// The exact source text this store last persisted, per document. The
    /// in-app up-loop's echo test: a watcher event whose file still holds
    /// the last persist is the room's own write coming back around, and
    /// reconciling it into the room would revert whatever was typed since.
    last_saved: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

impl FileDocStore {
    pub fn new(index: Arc<DocIndex>) -> Arc<Self> {
        Arc::new(Self {
            index,
            paths: std::sync::RwLock::new(HashMap::new()),
            last_saved: std::sync::Mutex::new(std::collections::HashMap::new()),
        })
    }

    pub fn register(&self, index: &DocIndex) {
        let mut paths = self.paths.write().unwrap();
        for (id, _) in index.entries() {
            if let Some(path) = index.absolute(&id) {
                paths.insert(id, path);
            }
        }
    }

    /// Whether `text` is byte-for-byte what this store last wrote for `key`.
    pub fn was_own_write(&self, key: &str, text: &str) -> bool {
        self.last_saved
            .lock()
            .map(|m| m.get(key).is_some_and(|last| last == text))
            .unwrap_or(false)
    }

    /// Where the encoded CRDT state for a document lives.
    fn crdt_path(&self, key: &DocKey) -> PathBuf {
        let root = self
            .paths
            .read()
            .unwrap()
            .get(key)
            .and_then(|p| p.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| self.index.root().to_path_buf());
        root.join(".hick-cache")
            .join("crdt")
            .join(format!("{key}.bin"))
    }

    fn source_path(&self, key: &DocKey) -> Result<PathBuf> {
        if let Some(path) = self.paths.read().unwrap().get(key) {
            return Ok(path.clone());
        }
        self.index
            .absolute(key)
            .with_context(|| format!("no document with id {key} under the served directory"))
    }
}

#[async_trait]
impl DocStore for FileDocStore {
    async fn load_source(&self, key: &DocKey) -> Result<String> {
        let path = self.source_path(key)?;
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))
    }

    async fn load_crdt(&self, key: &DocKey) -> Result<Option<Vec<u8>>> {
        let path = self.crdt_path(key);
        match std::fs::read(&path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
        }
    }

    async fn save(&self, key: &DocKey, source: &str, crdt: &[u8]) -> Result<()> {
        let path = self.source_path(key)?;
        // Write the document through a temp file in the same directory and
        // rename: a host whose editor is watching this file must never see a
        // half-written document, and a crash mid-save must not truncate their
        // work to nothing.
        write_atomic(&path, source.as_bytes())
            .with_context(|| format!("writing {}", path.display()))?;
        if let Ok(mut m) = self.last_saved.lock() {
            m.insert(key.clone(), source.to_string());
        }

        let crdt_path = self.crdt_path(key);
        if let Some(parent) = crdt_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // The CRDT sidecar is a cache: losing it costs a re-seed, not data, so
        // a failure here is reported and never fails the document write.
        if let Err(e) = write_atomic(&crdt_path, crdt) {
            log::warn!("could not store CRDT state at {}: {e}", crdt_path.display());
        }
        Ok(())
    }
}

/// Write `bytes` to `path` via a sibling temp file and a rename.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write as _;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension(format!(
        "{}.tmp",
        path.extension().and_then(|e| e.to_str()).unwrap_or("part")
    ));
    {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(bytes)?;
        if let Ok(metadata) = std::fs::metadata(path) {
            file.set_permissions(metadata.permissions())?;
        }
        file.sync_all()?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_index_excludes_runtime_documents_and_keeps_explicit_configuration() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("note.md"), "project note").unwrap();
        for folder in [
            ".hick-cache/adapters/tool",
            ".venv/lib/site-packages/package",
            "node_modules/pkg",
            "target/doc",
        ] {
            let path = dir.path().join(folder);
            std::fs::create_dir_all(&path).unwrap();
            std::fs::write(path.join("README.md"), "dependency").unwrap();
        }
        let entries = DocIndex::scan(dir.path()).unwrap().entries();
        assert_eq!(
            entries
                .iter()
                .map(|(_, path)| path.as_str())
                .collect::<Vec<_>>(),
            vec!["note.md"]
        );
        std::fs::write(
            dir.path().join("_hick.yml"),
            "files:\n  - .hick-cache/adapters/tool/README.md\n",
        )
        .unwrap();
        let entries = DocIndex::scan(dir.path()).unwrap().entries();
        assert_eq!(
            entries
                .iter()
                .map(|(_, path)| path.as_str())
                .collect::<Vec<_>>(),
            vec![".hick-cache/adapters/tool/README.md"]
        );
    }

    #[test]
    fn background_discovery_ignores_new_dependencies_but_keeps_explicit_documents_live() {
        let dir = tempfile::tempdir().unwrap();
        let index = DocIndex::scan(dir.path()).unwrap();
        std::fs::create_dir_all(dir.path().join(".venv/pkg")).unwrap();
        let dependency = ".venv/pkg/LICENSE.md";
        std::fs::write(dir.path().join(dependency), "dependency").unwrap();
        assert!(index.discover(dependency).is_none());
        assert!(index.entries().is_empty());
        std::fs::write(dir.path().join("new.md"), "source").unwrap();
        assert!(index.discover("new.md").is_some());
        let opened = index.add(dependency);
        assert_eq!(index.discover(dependency), Some(opened));
        let configured = ".venv/pkg/configured.md";
        std::fs::write(dir.path().join(configured), "chosen").unwrap();
        std::fs::write(
            dir.path().join("_hick.yml"),
            "files:\n  - .venv/pkg/configured.md\n",
        )
        .unwrap();
        assert!(index.discover(configured).is_some());
        assert!(index.discover("not-configured.md").is_none());
    }

    #[test]
    fn ids_are_stable_and_path_derived() {
        // A share link has to survive a restart, so this must not be random.
        assert_eq!(doc_id("docs/tour.md"), doc_id("docs/tour.md"));
        assert_ne!(doc_id("docs/tour.md"), doc_id("docs/other.md"));
        assert_eq!(doc_id("a.md").len(), 16);
    }

    #[tokio::test]
    async fn the_store_round_trips_a_document_and_its_crdt_state() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.md"), "original\n").unwrap();
        let index = Arc::new(DocIndex::scan(dir.path()).unwrap());
        let (id, rel) = index.sole().unwrap();
        assert_eq!(rel, "a.md");

        let store = FileDocStore::new(index);
        assert_eq!(store.load_source(&id).await.unwrap(), "original\n");
        assert_eq!(store.load_crdt(&id).await.unwrap(), None);

        store.save(&id, "edited\n", b"crdt-bytes").await.unwrap();
        // The edit landed in the user's actual file — that is the whole point.
        assert_eq!(
            std::fs::read_to_string(dir.path().join("a.md")).unwrap(),
            "edited\n"
        );
        assert_eq!(
            store.load_crdt(&id).await.unwrap().as_deref(),
            Some(b"crdt-bytes".as_slice())
        );
        // …and no stray temp files were left beside it.
        let names: Vec<String> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        assert!(!names.iter().any(|n| n.contains("tmp")), "{names:?}");
    }

    #[tokio::test]
    async fn an_unknown_id_names_what_went_wrong() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.md"), "x").unwrap();
        let index = Arc::new(DocIndex::scan(dir.path()).unwrap());
        let store = FileDocStore::new(index);
        let err = store
            .load_source(&"deadbeef".to_string())
            .await
            .unwrap_err();
        assert!(err.to_string().contains("no document with id"), "{err}");
    }
}
</book:entry>

</book:doc>
